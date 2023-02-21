use clap::Parser;
use dashmap::DashMap;
use handler::handle;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use routing::{Endpoint, RoutingPrecedentMap};
use tokio::time::{self, Instant};

use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use crate::hub::{hub_healthcheck_thread, Hub};

mod error;
mod handler;
mod hub;
mod routing;
mod schema;

#[derive(clap::Parser, Debug)]

struct Args {
    #[arg(long, default_value_t = 6543)]
    bindport: u16,

    #[arg(long, default_value_t = IpAddr::from_str("0.0.0.0").unwrap())]
    bindip: IpAddr,

    #[arg(long, default_value_t = 4)]
    num_threads: usize,

    /// An ip:port endpoint to route to.
    /// Can be used multiple times.
    /// example: --endpoint 192.168.1.1:1234 --endpoint 192.168.1.2:4321
    #[arg(long, value_parser=parse_ip)]
    endpoint: Vec<Endpoint>,
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

#[tokio::main]
async fn main() {
    let args = Args::parse();

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
    let hubs = Arc::new(_hubs);

    // Spawn the healthcheck thread
    tokio::task::spawn({
        let hub_clone = hubs.clone();
        async move { hub_healthcheck_thread(hub_clone).await }
    });

    println!("Binding on {}:{}", args.bindip, args.bindport);
    // Construct our SocketAddr to listen on...
    let addr = SocketAddr::from((args.bindip, args.bindport));
    // And a MakeService to handle each connection...
    let session_id_to_endpoint: Arc<RoutingPrecedentMap> = Arc::new(DashMap::new());

    // Spawn reaper thread for dead threads sitting around in the queue
    tokio::task::spawn({
        let map_clone = session_id_to_endpoint.clone();
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
    let server = Server::bind(&addr).serve(make_service_fn(move |_con| {
        let map = session_id_to_endpoint.clone();
        let hub_map = hubs.clone();
        async {
            Ok::<_, Infallible>(service_fn(move |_conn| {
                handle(_conn, map.clone(), hub_map.clone())
            }))
        }
    }));

    // And run forever...
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
