use crate::conf::{API_BIND_IP, API_BIND_PORT};
use crate::hub::Hub;
use crate::routing::{Endpoint, RoutingDecision};
use crate::schema::{HubExternal, Session};
use dashmap::DashMap;
use hyper::StatusCode;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use warp::{reply, Filter};

/// Primary entrypoint for the API. Will run and provide information and capabilities to
/// update information on the running Hub programatically.
pub async fn hub_api_thread(
    hubs: Arc<DashMap<Endpoint, Hub>>,
    sessions: Arc<DashMap<String, RoutingDecision>>,
    config: Arc<config::Config>,
) {
    let hubs_filter = warp::any().map(move || hubs.clone());
    let sessions_filter = warp::any().map(move || sessions.clone());

    let get_hubs = warp::get()
        .and(warp::path!("api" / "hubs"))
        .and(warp::path::end())
        .and(hubs_filter.clone())
        .and_then(get_hubs);

    let create_hub = warp::post()
        .and(warp::path!("api" / "hubs"))
        .and(warp::path::end())
        .and(hubs_filter.clone())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<Vec<HubExternal>>())
        .and_then(create_hub);

    let delete_hub = warp::delete()
        .and(warp::path!("api" / "hubs" / String))
        .and(warp::path::end())
        .and(hubs_filter.clone())
        .and_then(delete_hub);

    let update_hub = warp::patch()
        .and(warp::path!("api" / "hubs"))
        .and(warp::path::end())
        .and(hubs_filter.clone())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json::<HubExternal>())
        .and_then(update_hub);

    let get_sessions = warp::get()
        .and(warp::path!("api" / "sessions"))
        .and(warp::path::end())
        .and(sessions_filter.clone())
        .and_then(get_sessions);

    let routes = get_hubs
        .or(create_hub)
        .or(delete_hub)
        .or(update_hub)
        .or(get_sessions)
        .or(warp::any().map(|| {
            Ok(warp::reply::with_status(
                reply::reply(),
                StatusCode::NOT_FOUND,
            ))
        }));

    warp::serve(routes)
        .run(config_to_tuple(config.as_ref()))
        .await;
}

async fn get_hubs(hubs: Arc<DashMap<Endpoint, Hub>>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut lhubs: Vec<Hub> = vec![];

    for hub in hubs.iter() {
        let mut c = hub.clone();
        lhubs.push(c);
    }

    Ok(warp::reply::json(&lhubs))
}

async fn create_hub(
    hubs: Arc<DashMap<Endpoint, Hub>>,
    new_hubs: Vec<HubExternal>,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("creating new hub...");

    for new_hub in new_hubs {
        let ip = new_hub.ip;
        let port = new_hub.port;
        if hubs.contains_key(&(new_hub.ip, new_hub.port)) {
            return Ok(warp::reply::with_status(
                format!("hub at {}:{} already registered", ip, port),
                StatusCode::NOT_ACCEPTABLE,
            ));
        } else {
            hubs.insert(
                (new_hub.ip, new_hub.port),
                Hub::new_with_name(new_hub.name, new_hub.ip, new_hub.port),
            );
        }
    }

    return Ok(warp::reply::with_status(
        format!("registered hubs successfully"),
        StatusCode::CREATED,
    ));
}

async fn delete_hub(
    name: String,
    hubs: Arc<DashMap<Endpoint, Hub>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut key: Option<(IpAddr, u16)> = None;

    for hub in hubs.iter_mut() {
        if hub.name == name {
            key = Some(hub.key().clone());
        }
    }

    if let Some(key) = key {
        hubs.remove(&key);
        return Ok(warp::reply::reply());
    } else {
        Ok(warp::reply::reply())
        // return Ok(warp::reply::with_status(
        //     format!("hub at {}:{} already registered", ip, port),
        //     StatusCode::NOT_ACCEPTABLE,
        // ));
    }
}

async fn update_hub(
    hubs: Arc<DashMap<Endpoint, Hub>>,
    new_hub: HubExternal,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Ehh, for now just ietrate through all of the Hubs until we find a match, there
    // should never be enough where it would be an issue anyways.

    println!("updating hub now...");

    for mut hub in hubs.iter_mut() {
        if hub.name == new_hub.name {
            hub.ip = new_hub.ip;
            hub.port = new_hub.port;
            // hub.username = new_hub.username.clone();
            // hub.password = new_hub.password.clone();

            println!("hub updated");

            return Ok(warp::reply::reply());
        } else {
            continue;
        }
    }

    // Ok(warp::reply::with_status(
    //     format!(""),
    //     StatusCode::NOT_ACCEPTABLE,
    // ))

    Err(warp::reject())
}

async fn get_sessions(
    sessions: Arc<DashMap<String, RoutingDecision>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut sess: Vec<Session> = vec![];

    for s in sessions.iter() {
        sess.push(Session::new(s.key(), s.endpoint));
    }

    Ok(warp::reply::json(&sess))
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
