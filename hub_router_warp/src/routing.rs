use dashmap::{mapref::multiple::RefMulti, DashMap};
use hyper::{Body, Request, Uri};
use std::{net::IpAddr, sync::Arc, time::SystemTime};
use tokio::time::Instant;

use crate::{
    error::{HubRouterError, RoutingError},
    hub::{Hub, HubReadiness},
    schema::NewSessionRequestCapability,
};

pub type RoutingPrecedentMap = DashMap<String, RoutingDecision>;
pub type Endpoint = (IpAddr, u16);

#[derive(Debug)]
pub struct RoutingDecision {
    pub endpoint: Endpoint,
    pub decision_time: Instant,
}

impl RoutingDecision {
    pub fn new(endpoint: Endpoint, decision_time: Instant) -> RoutingDecision {
        RoutingDecision {
            endpoint,
            decision_time,
        }
    }
}

pub fn make_routing_decision(
    maybe_session_id: Option<String>,
    optional_requested_capabilities: Option<Vec<NewSessionRequestCapability>>,
    routing_precedent_map: Arc<RoutingPrecedentMap>,
    endpoint_state_map: Arc<DashMap<Endpoint, Hub>>,
) -> Result<Endpoint, RoutingError> {
    if let Some(session_id) = &maybe_session_id {
        if let Some(decision) = routing_precedent_map.get(session_id) {
            return Ok(decision.endpoint);
        } else {
            println!("Got a sessionID who hasn't been assigned anything");
        }
    }

    println!("Making a routing decision for the first time!");

    let mut healthy_hubs = endpoint_state_map
        .iter()
        .filter(|h| h.readiness == HubReadiness::Ready)
        .peekable();

    if healthy_hubs.peek().is_none() {
        return Err(RoutingError::NoHealthyNodes(
            "There are no healthy hubs to serve requests to".to_string(),
        ));
    }

    // get a list of hubs which satisfy the request
    let potential_hubs = match &optional_requested_capabilities {
        None => Some(healthy_hubs.collect()),
        Some(requested_capabilities) => {
            let mut satisfying_hubs: Option<Vec<RefMulti<Endpoint, Hub>>> = None;
            for capability in requested_capabilities {
                let is_satisfiable =
                    healthy_hubs.any(|pair| pair.value().can_satisfy_capability(&capability));
                if is_satisfiable {
                    satisfying_hubs = Some(
                        healthy_hubs
                            .filter(|pair| pair.value().can_satisfy_capability(&capability))
                            .collect(),
                    );
                    break;
                }
            }
            satisfying_hubs
        }
    };

    match potential_hubs {
        Some(hubs) => {
            // Make a routing decision
            // TODO: do better than random
            let first = hubs
                .iter()
                .nth(
                    (SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()) as usize
                        % hubs.len(),
                )
                .unwrap();
            let decision = (first.ip.clone(), first.port.clone());
            // Save it if we have a session id
            if let Some(session_id) = maybe_session_id {
                routing_precedent_map.insert(
                    session_id.to_string(),
                    RoutingDecision::new(decision, Instant::now()),
                );
            }
            return Ok(decision);
        }
        None => {
            return Err(RoutingError::UnableToSatisfyCapabilities(String::from(
                format!(
                    "No hubs could satisfy capabilities: {:?}",
                    &optional_requested_capabilities
                ),
            )));
        }
    }
}

pub fn apply_routing_decision(
    req: &mut Request<Body>,
    (decision_ip, decision_port): &Endpoint,
) -> Result<(), HubRouterError> {
    let path_string = match req.uri().path_and_query() {
        Some(p_q) => p_q.to_string(),
        None => {
            return Err(RoutingError::MalformedRequestPath(
                format!("Path was none for request URI {:?}", req.uri()).to_string(),
            )
            .into())
        }
    };

    let endpoint_uri_result = Uri::builder()
        .scheme("http")
        .authority(format!("{}:{}", decision_ip, decision_port))
        .path_and_query(path_string)
        .build();

    let endpoint_uri = match endpoint_uri_result {
        Ok(uri) => uri,
        Err(e) => {
            return Err(HubRouterError::Internal(format!(
                "Could not form endpoint_uri {:?}",
                e
            )))
        }
    };

    *req.uri_mut() = endpoint_uri;
    Ok(())
}
