use crate::api::hub_api_thread;
use crate::conf::{load_in_config, PROXY_BIND_IP, PROXY_BIND_PORT, HUBS_FILE_PATH};
use crate::hub::{hub_healthcheck_thread, Hub, HubMetadata};
use ::config::Config;
use clap::Parser;
use dashmap::DashMap;
use handler::handle;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use routing::{RoutingPrecedentMap};
use uuid::Uuid;
use std::convert::Infallible;
use std::fs::read_to_string;
#[allow(unused)]
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{self, Instant};

mod api;
mod conf;
mod error;
mod handler;
mod hub;
mod routing;
mod schema;
mod state;
mod utils;
mod ui;

#[derive(clap::Parser, Debug)]

struct Args {
    /// Location to read in configuration file from.
    #[arg(short, long, default_value_t = String::from("./config.yaml"))]
    config_location: String,

}


pub type HubMap = DashMap<Uuid, Hub>;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config: Config = match load_in_config(&args.config_location) {
        Ok(conf) => {
            conf
        }
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            return;
        }
    };
    let config = Arc::new(config);

    let hub_file_path = match config.get(HUBS_FILE_PATH) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Could not fetch hub file path from config: {}, defaulting to ./hubs.ser", e);
            "./hubs.ser".to_string()
        }
    };

    // Load serialized hubs from the hubs file, and create a DashMap
    let hubs: HubMap = match read_to_string(hub_file_path) {
        Ok(serialized_hubs) => {
            let deserialized_hubs: Result<Vec<HubMetadata>, _> = serde_json::from_str(&serialized_hubs);
            match deserialized_hubs {
                Ok(hubs) => {
                    let map: HubMap = DashMap::new();
                    hubs.into_iter().for_each(|h| {map.insert(h.uuid, Hub::from_meta(h));});
                    map
                },
                Err(e) => {
                    eprintln!("Error deserializing hubs: {}, no hubs will be initially registered", e);
                    DashMap::new()
                }
            }
        },
        Err(_) => {
            eprintln!("Could not open hubs file - no hubs will be initially registered");
            DashMap::new()
        }
    };
    let hubs = Arc::new(hubs);

    // Global session handler map.
    let sessions: Arc<RoutingPrecedentMap> = Arc::new(DashMap::new());

    // Spawn the healthcheck thread.
    tokio::task::spawn({
        let hub_clone = hubs.clone();
        async move { hub_healthcheck_thread(hub_clone).await }
    });

    // Spawn the API thread.
    tokio::task::spawn({
        let hub_clone = hubs.clone();
        let sessions_clone = sessions.clone();
        let config_clone = config.clone();
        async move { hub_api_thread(hub_clone, sessions_clone, config_clone).await }
    });

    // Spawn reaper thread for dead threads sitting around in the queue.
    tokio::task::spawn({
        let map_clone = sessions.clone();
        let mut reap_interval = time::interval(Duration::from_secs(60));
        let max_session_lifetime = Duration::from_secs(60 * 30);
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

    let bind_ip_str: String = match config.get(PROXY_BIND_IP) {
        Ok(str) => str,
        Err(e) => {
            eprintln!("Could not load proxy bind IP from config: {}, falling back to 0.0.0.0", e);
            "0.0.0.0".into()
        }
    };
    let bind_ip = match Ipv4Addr::from_str(&bind_ip_str) {
        Ok(ip) => ip,
        Err(e) => {
            eprintln!("Invalid proxy IP: {}", e);
            return;
        }
    };
    let bind_port = match config.get(PROXY_BIND_PORT){
        Ok(port) => port,
        Err(e) => {
            eprintln!("Could not load proxy port from config, falling back to 6543: {}", e);
            6543
        }
    };


    println!("Binding on {}:{}", bind_ip, bind_port);

    let server = Server::bind(&SocketAddr::from((bind_ip, bind_port))).serve(
        make_service_fn(move |_con| {
            let map = sessions.clone();
            let hub_map = hubs.clone();
            async {
                Ok::<_, Infallible>(service_fn(move |_conn| {
                    handle(_conn, map.clone(), hub_map.clone())
                }))
            }
        }),
    );

    // And run forever...
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
