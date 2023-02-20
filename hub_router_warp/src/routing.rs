use dashmap::DashMap;
use std::{net::IpAddr, sync::Arc, time::SystemTime};
use tokio::time::Instant;

use crate::hub::Hub;

pub type RoutingPrecedentMap = DashMap<String, RoutingDecision>;

#[derive(Debug)]
pub struct RoutingDecision {
    pub endpoint: (IpAddr, u16),
    pub decision_time: Instant,
}

impl RoutingDecision {
    pub fn new(endpoint: (IpAddr, u16), decision_time: Instant) -> RoutingDecision {
        RoutingDecision {
            endpoint,
            decision_time,
        }
    }
}

pub fn make_routing_decision(
    maybe_session_id: Option<String>,
    routing_precedent_map: Arc<RoutingPrecedentMap>,
    endpoint_state_map: Arc<DashMap<(IpAddr, u16), Hub>>,
) -> (IpAddr, u16) {
    if let Some(session_id) = &maybe_session_id {
        if let Some(decision) = routing_precedent_map.get(session_id) {
            return decision.endpoint;
        } else {
            println!("Got a sessionID who hasn't been assigned anything");
        }
    }

    println!("Making a routing decision for the first time!");

    // Make a routing decision
    let first = endpoint_state_map
        .iter()
        .nth(
            (SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                % 2) as usize
                % 2,
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
    return decision;
}
