use crate::conf::{API_BIND_IP, API_BIND_PORT};
use crate::hub::Hub;
use crate::routing::{Endpoint, RoutingDecision};
use dashmap::DashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use warp::Filter;

/// Primary entrypoint for the API. Will run and provide information and capabilities to
/// update information on the running Hub programatically.
pub async fn hub_api_thread(
    hubs: Arc<DashMap<Endpoint, Hub>>,
    sessions: Arc<DashMap<String, RoutingDecision>>,
    config: Arc<config::Config>,
) {
    let routes = warp::any().map(|| format!("hello world"));

    warp::serve(routes)
        .run(config_to_tuple(config.as_ref()))
        .await;
}

/// terrible function to parse contents from configuration and return the address to bind on.
fn config_to_tuple(configs: &config::Config) -> SocketAddr {
    if let Ok(ip) = configs.get_string(API_BIND_IP) {
        if let Ok(port) = configs.get::<u16>(API_BIND_PORT) {
            if let Ok(parsed) = IpAddr::from_str(&ip) {
                return SocketAddr::new(parsed, port);
            } else {
                return SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
            }
        } else {
            return SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
        }
    } else {
        return SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8080);
    }
}
