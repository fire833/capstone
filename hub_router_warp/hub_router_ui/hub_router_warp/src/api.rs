use crate::HubMap;
use crate::conf::{API_BIND_IP, API_BIND_PORT};
use crate::hub::Hub;
use crate::routing::{RoutingDecision};
use crate::schema::Session;
use crate::ui::WebUIAssets;
use dashmap::DashMap;
use hyper::StatusCode;
use log::info;
use uuid::Uuid;
use warp::path::Tail;
use warp::reply::Response;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use warp::{reply, Filter};

/// Primary entrypoint for the API. Will run and provide information and capabilities to
/// update information on the running Hub programatically.
pub async fn hub_api_thread(
    hubs: Arc<HubMap>,
    sessions: Arc<DashMap<String, RoutingDecision>>,
    config: Arc<config::Config>,
) {
    info!("starting api thread");
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
        .and(warp::body::json::<Hub>())
        .and_then(create_hub);

    let delete_hub = warp::delete()
        .and(warp::path!("api" / "hubs" / Uuid))
        .and(warp::path::end())
        .and(hubs_filter.clone())
        .and_then(delete_hub);

    let get_sessions = warp::get()
        .and(warp::path!("api" / "sessions"))
        .and(warp::path::end())
        .and(sessions_filter.clone())
        .and_then(get_sessions);

    let get_ui = warp::get()
        .and(warp::path("ui"))
        .and(warp::path::tail())
        .and_then(serve_ui);

    let routes = get_hubs
        .or(create_hub)
        .or(delete_hub)
        .or(get_sessions)
        .or(get_ui)
        .or(warp::any().map(|| {
            Ok(warp::reply::with_status(
                reply::reply(),
                StatusCode::NOT_FOUND,
            ))
        }))
        .with(warp::cors().allow_any_origin().allow_methods(vec!["GET", "POST", "OPTIONS"]));

    warp::serve(routes)
        .run(config_to_tuple(config.as_ref()))
        .await;
}

async fn get_hubs(hubs: Arc<HubMap>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut lhubs: Vec<Hub> = vec![];

    for hub in hubs.iter() {
        let c = hub.clone();
        lhubs.push(c);
    }

    Ok(warp::reply::json(&lhubs))
}

async fn create_hub(
    hubs: Arc<HubMap>,
    new_hub: Hub,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("creating new hub...");

    let uuid = &new_hub.meta.uuid;
    if hubs.contains_key(uuid) {
        return Ok(warp::reply::with_status(
            format!("hub with id {} already registered", &uuid),
            StatusCode::NOT_ACCEPTABLE,
        ));
    } else {
        hubs.insert(uuid.clone(), new_hub);
    }

    return Ok(warp::reply::with_status(
        format!("registered hubs successfully"),
        StatusCode::CREATED,
    ));
}

async fn delete_hub(
    uuid: Uuid,
    hubs: Arc<HubMap>,
) -> Result<impl warp::Reply, warp::Rejection> {
    hubs.remove(&uuid);
    Ok(warp::reply::reply())
}

async fn get_sessions(
    sessions: Arc<DashMap<String, RoutingDecision>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut sess: Vec<Session> = vec![];

    for s in sessions.iter() {
        sess.push(Session::new(s.key(), &s.value().hub_endpoint));
    }

    Ok(warp::reply::json(&sess))
}

async fn serve_ui(tail: Tail) -> Result<impl warp::Reply, warp::Rejection>  {
    let path = if tail.as_str() == "" { "index.html" } else { tail.as_str() };

    let maybe_file = WebUIAssets::get(path);
    match maybe_file {
        Some(file) => {
            Ok(
                warp::reply::with_header(Response::new(file.data), "Content-Type", mime_guess::from_path(path).first_or_octet_stream().to_string())
            )
        },
        None => {
            Err(
                warp::reject::not_found()
            )
        },
    }
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
