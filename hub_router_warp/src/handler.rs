use dashmap::DashMap;
use hyper::{Body, Client, Method, Request, Response, Uri};
use lazy_static::lazy_static;
use regex::Regex;
use std::{net::IpAddr, sync::Arc};
use tokio::time::Instant;

use crate::{
    hub::Hub,
    routing::{make_routing_decision, RoutingDecision, RoutingPrecedentMap},
    schema::NewSessionResponse,
};

fn extract_sessionid(req: &Request<Body>) -> Option<String> {
    lazy_static! {
        static ref SESSION_ID_REGEXP: Regex =
            Regex::new(r"^/session/([a-zA-Z0-9]*)(/|\z)").unwrap();
    }

    match req.uri().path_and_query() {
        None => return None,
        Some(path_and_query) => {
            let path_string = path_and_query.path();
            let first_capture = SESSION_ID_REGEXP.captures_iter(path_string).next();
            match first_capture {
                None => return None,
                Some(captures) => match captures.get(1) {
                    None => {
                        eprintln!(
                            "SessionID regexp found a match but it didn't include the sessionID"
                        );
                        return None;
                    }
                    Some(session_id_match) => return Some(session_id_match.as_str().to_string()),
                },
            }
        }
    }
}

fn is_request_new_session(req: &Request<Body>) -> bool {
    return req.method() == Method::POST
        && req.uri().path_and_query().is_some()
        && req.uri().path_and_query().unwrap().as_str() == "/session";
}

fn is_delete_session(req: &Request<Body>) -> bool {
    return req.method() == Method::DELETE
        && req.uri().path_and_query().is_some()
        && req
            .uri()
            .path_and_query()
            .unwrap()
            .as_str()
            .starts_with("/session/");
}

async fn handle_new_session_request(
    req: Request<Body>,
    routing_map: Arc<RoutingPrecedentMap>,
    _endpoint_map: Arc<DashMap<(IpAddr, u16), Hub>>,
    routing_decision: (IpAddr, u16),
) -> Result<Response<Body>, hyper::Error> {
    let client = Client::new();
    let result = client.request(req).await;

    match result {
        Ok(response) => {
            let (parts, body) = response.into_parts();
            let bytes_result = hyper::body::to_bytes(body).await;
            match bytes_result {
                Err(e) => {
                    eprintln!("Error converting body buffer: {}", e);
                    Err(e)
                }
                Ok(bytes) => {
                    let parsed: Result<NewSessionResponse, _> = serde_json::from_slice(&bytes);
                    match parsed {
                        Ok(new_session_response) => {
                            println!("Making a routing decision from the session response");
                            let session_id = new_session_response.value.sessionId;
                            routing_map.insert(
                                session_id,
                                RoutingDecision::new(routing_decision, Instant::now()),
                            );
                        }
                        Err(parse_error) => {
                            eprintln!(
                                "Error parsing NewSessionResponse: {} | {}",
                                parse_error,
                                String::from_utf8_lossy(&bytes)
                            )
                        }
                    }
                    Ok(hyper::Response::from_parts(parts, hyper::Body::from(bytes)))
                }
            }
        }
        Err(e) => Err(e),
    }
}

async fn forward_request(
    req: Request<Body>,
    _routing_map: Arc<RoutingPrecedentMap>,
    _endpoint_map: Arc<DashMap<(IpAddr, u16), Hub>>,
) -> Result<Response<Body>, hyper::Error> {
    let client = Client::new();
    client.request(req).await
}

pub async fn handle(
    req: Request<Body>,
    routing_map: Arc<RoutingPrecedentMap>,
    endpoint_map: Arc<DashMap<(IpAddr, u16), Hub>>,
) -> Result<Response<Body>, hyper::Error> {
    println!("Got req uri: {:#?}", req.uri());

    let maybe_session_id = extract_sessionid(&req);

    let (ip, port) = make_routing_decision(
        maybe_session_id.clone(),
        routing_map.clone(),
        endpoint_map.clone(),
    );

    let path_string = match req.uri().path_and_query() {
        Some(p_q) => p_q.to_string(),
        None => {
            panic!("Request had no path: {:#?}", req);
        }
    };

    println!("Got routing decision: {}:{}", ip, port);
    let endpoint_uri_result = Uri::builder()
        .scheme("http")
        .authority(format!("{}:{}", ip, port))
        .path_and_query(path_string)
        .build();

    let endpoint_uri = match endpoint_uri_result {
        Ok(uri) => uri,
        Err(e) => {
            panic!("Could not create endpoint uri: {}", e);
        }
    };

    let mut req = req;
    *req.uri_mut() = endpoint_uri;

    if is_request_new_session(&req) {
        // handle new sessions differently
        return handle_new_session_request(req, routing_map, endpoint_map, (ip, port)).await;
    } else if is_delete_session(&req) && maybe_session_id.is_some() {
        routing_map.remove(&maybe_session_id.unwrap());
    }
    return forward_request(req, routing_map, endpoint_map).await;
}
