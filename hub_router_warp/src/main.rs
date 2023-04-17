use crate::api::hub_api_thread;
use crate::hub::{hub_healthcheck_thread, Hub};
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

#[derive(clap::Parser, Debug)]

struct Args {
    /// Location to read in configuration file from.
    #[arg(short, long, default_value_t = String::from("./config.json"))]
    config_location: String,
}

pub type HubMap = DashMap<Uuid, Hub>;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let state: Arc<HubRouterState> = Arc::new(HubRouterState::new_from_disk(&args.config_location));

    // Global session handler map.
    let sessions: Arc<RoutingPrecedentMap> = Arc::new(DashMap::new());

    // Spawn the healthcheck thread.
    tokio::task::spawn({
        let state_clone = state.clone();
        async move { hub_healthcheck_thread(state_clone).await }
    });

    // Spawn the API thread.
    tokio::task::spawn({
        let state_clone = state.clone();
        let sessions_clone = sessions.clone();
        async move { hub_api_thread(state_clone, sessions_clone).await }
    });

    // Spawn reaper thread for dead threads sitting around in the queue.
    tokio::task::spawn({
        let map_clone = sessions.clone();
        let state_clone = state.clone();
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
        // let max_session_lifetime = Duration::from_secs(60 * state_clone);
        async move {
            loop {
                reap_interval.tick().await;
                println!("Map: {:#?}", map_clone);

                let dead_session_ids: Vec<String> = map_clone
                    .iter()
                    .filter(|entry| {
                        println!(
                            "Session id: {} has duration since: {:#?}",
                            entry.key(),
                            Instant::now().duration_since(entry.value().decision_time)
                        );
                        Instant::now()
                            .duration_since(entry.value().decision_time)
                            .ge(&max_session_lifetime)
                    })
                    .map(|e| e.key().clone())
                    .collect();
                println!("Culling dead sessions: {:#?}", dead_session_ids);
                dead_session_ids.iter().for_each(|key| {
                    map_clone.remove(key);
                });
            }
        }
    });

    let bind_addr = match state.configs.read() {
        Ok(conf) => SocketAddr::new(IpAddr::V4(conf.bind_ip), conf.bind_port),
        Err(e) => {
            warn!("RWLock poisoned generating proxy bind addr: {}", e);
            let conf = HubRouterPrimitiveConfigs::default();
            SocketAddr::new(IpAddr::V4(conf.bind_ip), conf.bind_port)
        }
    };

    println!("Binding on {}", bind_addr);

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
        eprintln!("server error: {}", e);
    }
}
