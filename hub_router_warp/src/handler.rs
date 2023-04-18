use crate::{
    error::HubRouterError,
    routing::{apply_routing_decision, make_routing_decision, RoutingPrecedentMap},
    schema::{NewSessionRequestBody, NewSessionRequestCapability, NewSessionResponse},
    state::HubRouterState,
};
use hyper::{Body, Client, Method, Request, Response};
use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use std::sync::Arc;

fn extract_session_id(req: &Request<Body>) -> Option<String> {
    lazy_static! {
        static ref SESSION_ID_REGEXP: Regex = Regex::new(r"^/session/([^/]*)(/|\z)").unwrap();
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
                        warn!(
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

#[test]
fn test_extract_session_id() {
    let t1 = hyper::Request::post("https://example.com/session/234787498dfhgshdfgsdhf0943/")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        extract_session_id(&t1),
        Some(String::from("234787498dfhgshdfgsdhf0943"))
    );

    let t2 = hyper::Request::get("https://example.com/sess/")
        .body(Body::empty())
        .unwrap();
    assert_eq!(extract_session_id(&t2), None);

    let t3 = hyper::Request::get("https://example.com/session/123456/3875285udfhd")
        .body(Body::empty())
        .unwrap();
    assert_eq!(extract_session_id(&t3), Some(String::from("123456")));

    let t4 = hyper::Request::get("https://example.com/session/*&*$&)HJSJKADHLAKDFS)_/3875285udfhd")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        extract_session_id(&t4),
        Some(String::from("*&*$&)HJSJKADHLAKDFS)_"))
    );
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

#[test]
fn test_req_is_new_session() {
    let t1 = hyper::Request::post("https://example.com/session")
        .body(Body::empty())
        .unwrap();
    assert_eq!(is_request_new_session(&t1), true);

    let t2 = hyper::Request::post("https://example.com/session/247579fhsjkdfhdsjkfhsfg")
        .body(Body::empty())
        .unwrap();
    assert_eq!(is_request_new_session(&t2), false);

    let t3 = hyper::Request::get("https://example.com/session/247579fhsjkdfhdsjkfhsfg")
        .body(Body::empty())
        .unwrap();
    assert_eq!(is_request_new_session(&t3), false);

    let t4 = hyper::Request::post("https://example.com/sessi")
        .body(Body::empty())
        .unwrap();
    assert_eq!(is_request_new_session(&t4), false);
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

#[test]
fn test_delete_session() {
    let t1 = hyper::Request::delete("https://example.com/session/237598yfub398f4h8dufd")
        .body(Body::empty())
        .unwrap();
    assert_eq!(is_delete_session(&t1), true);

    let t2 = hyper::Request::post("https://example.com/session/237598yfub398f4h8dufd")
        .body(Body::empty())
        .unwrap();
    assert_eq!(is_delete_session(&t2), false);

    let t3 = hyper::Request::delete("http://example.com/")
        .body(Body::empty())
        .unwrap();
    assert_eq!(is_delete_session(&t3), false);
}

async fn handle_new_session_request(
    mut req: Request<Body>,
    routing_map: Arc<RoutingPrecedentMap>,
    state: Arc<HubRouterState>,
) -> Result<Response<Body>, HubRouterError> {
    let (requests, reconstructed_request) =
        extract_capabilities_from_new_session_request(req).await?;
    req = reconstructed_request;

    let routing_decision =
        make_routing_decision(None, Some(requests), routing_map.clone(), state.clone())?;

    apply_routing_decision(&mut req, &routing_decision.hub_endpoint)?;

    let client = Client::new();
    let response = client.request(req).await?;

    let (parts, body) = response.into_parts();
    let bytes = hyper::body::to_bytes(body).await?;

    let maybe_new_session_response: Result<NewSessionResponse, serde_json::Error> =
        serde_json::from_slice(&bytes);
    let new_session_response = match maybe_new_session_response {
        Ok(res) => res,
        Err(_) => {
            return Err(HubRouterError::SessionCreationError(format!("Could not create session (this is likely because the hub is overloaded, increasing the hub's resource limits may be helpful): {}", String::from_utf8_lossy(&bytes))))
        }
    };
    let session_id = new_session_response.value.sessionId;
    routing_map.insert(session_id.to_string(), routing_decision);
    Ok(hyper::Response::from_parts(parts, hyper::Body::from(bytes)))
}

async fn forward_request(
    mut req: Request<Body>,
    _routing_map: Arc<RoutingPrecedentMap>,
    state: Arc<HubRouterState>,
) -> Result<Response<Body>, HubRouterError> {
    let maybe_session_id = extract_session_id(&req);
    let routing_decision = make_routing_decision(maybe_session_id, None, _routing_map, state)?;
    apply_routing_decision(&mut req, &routing_decision.hub_endpoint)?;

    let client = Client::new();
    HubRouterError::wrap_err(client.request(req).await)
}

async fn handle_delete_session_request(
    req: Request<Body>,
    routing_map: Arc<RoutingPrecedentMap>,
    state: Arc<HubRouterState>,
) -> Result<Response<Body>, HubRouterError> {
    let session_id = extract_session_id(&req);
    let result = forward_request(req, routing_map.clone(), state).await;
    routing_map.remove(&session_id.unwrap());
    result
}

/// Primary handler function for forwarding requests onto downstream Hubs.
/// This function will be called by the primary listener for forwarding requests.
pub async fn handle(
    req: Request<Body>,
    routing_map: Arc<RoutingPrecedentMap>,
    state: Arc<HubRouterState>,
) -> Result<Response<Body>, hyper::Error> {
    let maybe_session_id = extract_session_id(&req);

    let response: Result<Response<Body>, HubRouterError> = async {
        if is_request_new_session(&req) {
            // handle new sessions differently
            return handle_new_session_request(req, routing_map, state).await;
        } else if is_delete_session(&req) && maybe_session_id.is_some() {
            return handle_delete_session_request(req, routing_map, state).await;
        }
        return forward_request(req, routing_map, state).await;
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
