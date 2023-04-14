use crate::{
    error::{HubRouterError, RoutingError},
    hub::{Hub, HubReadiness},
    schema::NewSessionRequestCapability, HubMap,
};
use dashmap::{mapref::multiple::RefMulti, DashMap};
use hyper::{Body, Request, Uri};
use rand::random;
use uuid::Uuid;
use std::{str::FromStr, sync::Arc};
use tokio::time::Instant;
use url::Url;

pub type RoutingPrecedentMap = DashMap<String, RoutingDecision>;
pub type Endpoint = Url;

#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub hub_uuid: Uuid,
    pub hub_endpoint: Url,
    pub decision_time: Instant,
}

impl RoutingDecision {
    pub fn new(hub_uuid: Uuid, hub_endpoint: Url, decision_time: Instant) -> RoutingDecision {
        RoutingDecision {
            hub_uuid,
            hub_endpoint,
            decision_time,
        }
    }
}

pub fn make_routing_decision(
    maybe_session_id: Option<String>,
    optional_requested_capabilities: Option<Vec<NewSessionRequestCapability>>,
    routing_map: Arc<RoutingPrecedentMap>,
    endpoint_map: Arc<HubMap>,
) -> Result<RoutingDecision, RoutingError> {
    if let Some(session_id) = &maybe_session_id {
        if let Some(decision) = routing_map.get(session_id) {
            return Ok(decision.clone());
        } else {
            println!("Got a sessionID who hasn't been assigned anything");
        }
    }

    println!("Making a routing decision for the first time!");

    let mut healthy_hubs_iter = endpoint_map
        .iter()
        .filter(|h| h.state.get_readiness() == HubReadiness::Ready)
        .peekable();

    if healthy_hubs_iter.peek().is_none() {
        return Err(RoutingError::NoHealthyNodes(
            "There are no healthy hubs to serve requests to".to_string(),
        ));
    }

    let healthy_hubs: Vec<_> = healthy_hubs_iter.collect();

    // get a list of hubs which satisfy the request
    let potential_hubs: Option<Vec<&RefMulti<Uuid, Hub>>> =
        match &optional_requested_capabilities {
            None => Some(healthy_hubs.iter().collect()),
            Some(requested_capabilities) => {
                let mut satisfying_hubs: Option<Vec<&RefMulti<Uuid, Hub>>> = None;
                for capability in requested_capabilities {
                    let can_satisfy: Vec<_> = healthy_hubs
                        .iter()
                        .filter(|h| h.can_satisfy_capability(capability))
                        .collect();
                    if can_satisfy.len() > 0 {
                        satisfying_hubs = Some(can_satisfy);
                        break;
                    }
                }
                satisfying_hubs
            }
        };

        
    match potential_hubs {
        Some(hubs) => {
            println!("Found {} healthy hubs to route to", hubs.len());
            let keys_and_weights: Vec<_> = hubs
                .iter()
                .map(|h| (h.key(), (100 - h.state.get_fullness() + 1) as u32))
                .collect();

            let weight_sum = keys_and_weights
                .iter()
                .fold(0, |acc: u32, (_, weight)| acc + weight);

            let selection_weight_distance: u32 = random::<u32>() % weight_sum;
            let mut accumulated_weight: u32 = 0;

            let mut selected_hub_uuid: Option<Uuid> = None;
            for (uuid, weight) in keys_and_weights {
                accumulated_weight += weight;
                if accumulated_weight >= selection_weight_distance {
                    selected_hub_uuid = Some(uuid.clone());
                    break;
                }
            }

            if selected_hub_uuid.is_none() {
                eprintln!("Weighted rolling never selected an endpoint - this shouldn't happen, falling back to uniform random");
                return Err(RoutingError::NoDecision("Internal Error | Weighted random routing was unable to select a hub".into()));
            }

            let decision_uuid = match selected_hub_uuid {
                Some(uuid) => uuid,
                None => {
                    return Err(RoutingError::NoDecision("Internal Error | Unable to select a hub for routing".into()));
                }
            };

            let decision_ref = match endpoint_map.get(&decision_uuid) {
                Some(hub) => hub,
                None => {
                    return Err(RoutingError::NoDecision("A hub was selected for routing, but could not be retrieved".into()));
                }
            };

            let decision = RoutingDecision::new(decision_ref.key().clone(), decision_ref.value().meta.url.clone(), Instant::now());
            println!("Made decision: {:#?}", decision);

            // Save it if we have a session id
            if let Some(session_id) = maybe_session_id {
                routing_map.insert(
                    session_id.to_string(),
                    decision.clone(),
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
    endpoint: &Endpoint,
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

    let endpoint_with_path = {
        let mut ep = endpoint.clone();
        ep.set_path(&path_string);
        ep
    };
    let endpoint_uri_result = Uri::from_str(endpoint_with_path.as_str());

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
