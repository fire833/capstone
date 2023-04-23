///! The Hub Router is a WebDriver spec-compliant intermediate node to route
///! Selenium tests between multiple grids.

use crate::api::hub_api_thread;
use crate::hub::{hub_healthcheck_thread, Hub};
use crate::logger::HubRouterLogger;
use crate::state::{HubRouterPrimitiveConfigs, HubRouterState};
use clap::Parser;
use dashmap::DashMap;
use handler::handle;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use log::warn;
use routing::RoutingPrecedentMap;
use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{self, Instant};
use uuid::Uuid;

mod api;
mod error;
mod handler;
mod hub;
mod routing;
mod schema;
mod state;
mod ui;
mod utils;
mod logger;

#[derive(clap::Parser, Debug)]

/// Args is the wrapper struct for arguments passed when invoking the binary.
/// `config_location` is the only parameter, which informs the Hub Router
/// of where the configuration file should be located.
struct Args {
    /// Location to read in configuration file from.
    #[arg(short, long, default_value_t = String::from("./config.json"))]
    config_location: String,
}

/// A HubMap stores all of hubs which have been registered,
/// identified by a runtime-unique UUID (it is permissible to change between application restarts).
pub type HubMap = DashMap<Uuid, Hub>;

#[tokio::main]
async fn main() {

    // Initialize our logging provider - this allows us to collect warnings and errors
    // and serve them from the API
    HubRouterLogger::init();

    // Parse out command line arguments (the config file location), and load that config file
    let args = Args::parse();
    let state: Arc<HubRouterState> = Arc::new(HubRouterState::new_from_disk(&args.config_location));

    // We store routing decisions in this globally shared hashmap
    // from Selenium session IDs to URLs.
    let sessions: Arc<RoutingPrecedentMap> = Arc::new(DashMap::new());

    // Spawn the healthcheck thread, which polls each registered Selenium hub
    // for its fullness for each browser and operating system,
    // so that we can calculate routing weights, and ensure that we only
    // route tests to healthy hubs.
    tokio::task::spawn({
        let state_clone = state.clone();
        async move { hub_healthcheck_thread(state_clone).await }
    });

    // Spawn the API thread, which serves configuration endpoints and the UI 
    tokio::task::spawn({
        let state_clone = state.clone();
        let sessions_clone = sessions.clone();
        async move { hub_api_thread(state_clone, sessions_clone).await }
    });

    // Spawn reaper thread for dead threads sitting around in the queue,
    // any threads which error during execution and don't perform cleanup
    // steps will be left in our RoutingPrecedentMap until the reaper
    // thread cleans them up.
    // The reaper will clean up any tests which have been in the map
    // for longer than the configurable timeout (30 minutes by default)
    tokio::task::spawn({
        // Clone the Arcs so that the thread has its own local copy
        let map_clone = sessions.clone();
        let state_clone = state.clone();

        // Pull the reap interval and maximum session length from configuration,
        // and turn the integer seconds into a tokio interval and a duration 
        let (mut reap_interval, max_session_lifetime) = match state_clone.configs.read() {
            Ok(conf) => (
                time::interval(Duration::from_secs(conf.reaper_thread_interval.into())),
                Duration::from_secs(60 * conf.reaper_thread_duration_max),
            ),
            Err(e) => {
                warn!("Config Rwlock was poisoned during reaper spawn: {}", e);
                let conf = HubRouterPrimitiveConfigs::default();
                (
                    time::interval(Duration::from_secs(conf.reaper_thread_interval.into())),
                    Duration::from_secs(60 * conf.reaper_thread_duration_max),
                )
            }
        };


        // Core reaper loop - wait for the interval, then remove all sessions
        // whose age is greater than the max session lifetime
        async move {
            loop {
                reap_interval.tick().await;

                let dead_session_ids: Vec<String> = map_clone
                    .iter()
                    .filter(|entry| {
                        Instant::now()
                            .duration_since(entry.value().decision_time)
                            .ge(&max_session_lifetime)
                    })
                    .map(|e| e.key().clone())
                    .collect();
                dead_session_ids.iter().for_each(|key| {
                    map_clone.remove(key);
                });
            }
        }
    });

    // Extract the IP and port specified in config, and turn them into a SocketAddr ready for binding
    let bind_addr = match state.configs.read() {
        Ok(conf) => SocketAddr::new(IpAddr::V4(conf.bind_ip), conf.bind_port),
        Err(e) => {
            warn!("RWLock poisoned generating proxy bind addr: {}", e);
            let conf = HubRouterPrimitiveConfigs::default();
            SocketAddr::new(IpAddr::V4(conf.bind_ip), conf.bind_port)
        }
    };

    // Bind the request router on that SocketAddr
    let server = Server::bind(&bind_addr).serve(make_service_fn(move |_con| {
        let map = sessions.clone();
        let state_clone = state.clone();
        async {
            Ok::<_, Infallible>(service_fn(move |_conn| {
                handle(_conn, map.clone(), state_clone.clone())
            }))
        }
    }));

    // And run forever...
    if let Err(e) = server.await {
        warn!("server error: {}", e);
    }
}
