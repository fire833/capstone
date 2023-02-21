use dashmap::DashMap;
use hyper::{Body, Client, Method, Request, Response};
use lazy_static::lazy_static;
use regex::Regex;
use std::sync::Arc;
use tokio::time::Instant;

use crate::{
    error::HubRouterError,
    hub::Hub,
    routing::{
        apply_routing_decision, make_routing_decision, Endpoint, RoutingDecision,
        RoutingPrecedentMap,
    },
    schema::{NewSessionRequestBody, NewSessionRequestCapability, NewSessionResponse},
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

pub async fn extract_capabilities_from_new_session_request(
    request: Request<Body>,
) -> Result<(Vec<NewSessionRequestCapability>, Request<Body>), HubRouterError> {
    let (parts, body) = request.into_parts();
    let body_bytes = hyper::body::to_bytes(body).await?;

    let capability_request: NewSessionRequestBody = serde_json::from_slice(&body_bytes)?;
    let possible_requests = generate_possible_capabilities(capability_request);

    let reconstructed_request = hyper::Request::from_parts(parts, hyper::Body::from(body_bytes));
    return Ok((possible_requests, reconstructed_request));
}

fn generate_possible_capabilities(
    capability_request: NewSessionRequestBody,
) -> Vec<NewSessionRequestCapability> {
    capability_request
        .capabilities
        .firstMatch
        .iter()
        .map(|first_match| {
            let mut base = capability_request.capabilities.alwaysMatch.clone();
            base.browserName = base.browserName.or(first_match.browserName.clone());
            base.platformName = base.platformName.or(first_match.platformName.clone());
            base
        })
        .collect()
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
    mut req: Request<Body>,
    routing_map: Arc<RoutingPrecedentMap>,
    endpoint_map: Arc<DashMap<Endpoint, Hub>>,
) -> Result<Response<Body>, HubRouterError> {
    let (requests, reconstructed_request) =
        extract_capabilities_from_new_session_request(req).await?;
    req = reconstructed_request;

    let (ip, port) = make_routing_decision(
        None,
        Some(requests),
        routing_map.clone(),
        endpoint_map.clone(),
    )?;

    apply_routing_decision(&mut req, &(ip, port))?;

    let client = Client::new();
    let response = client.request(req).await?;

    let (parts, body) = response.into_parts();
    let bytes = hyper::body::to_bytes(body).await?;
    println!(
        "Got new session response: {}",
        String::from_utf8_lossy(&bytes)
    );
    let new_session_response: NewSessionResponse = serde_json::from_slice(&bytes)?;
    let session_id = new_session_response.value.sessionId;
    routing_map.insert(session_id, RoutingDecision::new((ip, port), Instant::now()));
    Ok(hyper::Response::from_parts(parts, hyper::Body::from(bytes)))
}

async fn forward_request(
    mut req: Request<Body>,
    _routing_map: Arc<RoutingPrecedentMap>,
    _endpoint_map: Arc<DashMap<Endpoint, Hub>>,
) -> Result<Response<Body>, HubRouterError> {
    let maybe_session_id = extract_sessionid(&req);
    let routing_decision =
        make_routing_decision(maybe_session_id, None, _routing_map, _endpoint_map)?;
    apply_routing_decision(&mut req, &routing_decision)?;

    let client = Client::new();
    HubRouterError::wrap_err(client.request(req).await)
}

pub async fn handle(
    req: Request<Body>,
    routing_map: Arc<RoutingPrecedentMap>,
    endpoint_map: Arc<DashMap<Endpoint, Hub>>,
) -> Result<Response<Body>, hyper::Error> {
    println!("Got req uri: {:#?}", req.uri());
    let maybe_session_id = extract_sessionid(&req);

    let response: Result<Response<Body>, HubRouterError> = async {
        if is_request_new_session(&req) {
            // handle new sessions differently
            return handle_new_session_request(req, routing_map, endpoint_map).await;
        } else if is_delete_session(&req) && maybe_session_id.is_some() {
            routing_map.remove(&maybe_session_id.unwrap());
        }
        return forward_request(req, routing_map, endpoint_map).await;
    }
    .await;

    // TODO: General Error Handling
    match response {
        Ok(response) => Ok(response),
        Err(e) => Ok(Response::builder()
            .status(500)
            .body(Body::from(format!("Hub Router error: {:#?}", e)))
            .unwrap()),
    }
}
