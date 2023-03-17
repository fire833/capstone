use clap::Parser;
use dashmap::DashMap;
use handler::handle;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use routing::{Endpoint, RoutingPrecedentMap};
use std::convert::Infallible;

use ::config::Config;

#[allow(unused)]
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{self, Instant};

use crate::api::hub_api_thread;
use crate::conf::load_in_config;
use crate::hub::{hub_healthcheck_thread, Hub};

mod api;
mod conf;
mod error;
mod handler;
mod hub;
mod routing;
mod schema;

#[derive(clap::Parser, Debug)]

struct Args {
    #[arg(long, default_value_t = 6543)]
    bind_port: u16,

    #[arg(long, default_value_t = IpAddr::from_str("0.0.0.0").unwrap())]
    bind_ip: IpAddr,

    /// An ip:port endpoint to route to.
    /// Can be used multiple times.
    /// example: --endpoint 192.168.1.1:1234 --endpoint 192.168.1.2:4321
    #[arg(long, value_parser=parse_ip)]
    endpoint: Vec<Endpoint>,

    /// Location to read in configuration file from.
    #[arg(short, long, default_value_t = String::from("/etc/router_warp/config.yaml"))]
    config_location: String,
}

fn parse_ip(s: &str) -> Result<Endpoint, String> {
    let mut split = s.split(":");
    let split_ip = split.next();
    let split_port = split.next();

    let ip_parse_result = match split_ip {
        Some(ip_string) => IpAddr::from_str(ip_string),
        None => return Err("Invalid IP address given".to_string()),
    };

    let port_parse_result = match split_port {
        Some(port_str) => port_str.parse::<u16>(),
        None => return Err("No port was given".to_string()),
    };

    let ip_addr = match ip_parse_result {
        Ok(addr) => addr,
        Err(e) => return Err(format!("Error parsing endpoint ip: {}", e)),
    };

    let port_u16 = match port_parse_result {
        Ok(u16) => u16,
        Err(e) => return Err(format!("Error parsing endpoint port: {}", e)),
    };

    Ok((ip_addr, port_u16))
}

#[test]
fn test_parse_ip() {
    let t1 = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    assert_eq!(Ok((t1, 5555)), parse_ip("127.0.0.1:5555"));

    let t2 = IpAddr::V4(Ipv4Addr::new(10, 50, 1, 1));
    assert_eq!(Ok((t2, 32456)), parse_ip("10.50.1.1:32456"));
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut _config: Config;

    match load_in_config(&args.config_location) {
        Ok(conf) => {
            _config = conf;
        }
        Err(e) => {
            println!("{}", e);
            return;
        }
    }

    println!("Got endpoints: {:#?}", args.endpoint);

    // Parse out the hubs from the args and organize them in a DashMap
    let _hubs_list: Vec<Hub> = args
        .endpoint
        .iter()
        .map(|(ip, port)| Hub::new(ip.clone(), port.clone()))
        .collect();
    let _hubs: DashMap<Endpoint, Hub> = DashMap::new();
    for h in _hubs_list {
        _hubs.insert((h.ip, h.port), h);
    }

    // Global config.
    let config = Arc::new(_config);

    // Global hubs list.
    let hubs = Arc::new(_hubs);

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

    println!("Binding on {}:{}", args.bind_ip, args.bind_port);

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

    // TODO: make sense of this utter mess
    let server = Server::bind(&SocketAddr::from((args.bind_ip, args.bind_port))).serve(
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
