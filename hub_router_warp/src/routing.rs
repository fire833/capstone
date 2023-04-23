//! Contains logic for making and remembering routing decisions

use crate::{
    error::{HubRouterError, RoutingError},
    hub::{Hub, HubReadiness},
    schema::NewSessionRequestCapability,
    state::HubRouterState,
};
use dashmap::{mapref::multiple::RefMulti, DashMap};
use hyper::{Body, Request, Uri};
use log::warn;
use rand::random;
use std::{str::FromStr, sync::Arc};
use tokio::time::Instant;
use url::Url;
use uuid::Uuid;

/// A map from Selenium session IDs to URLs, to remember previous routing decisions,
/// and forward later requests for that session to the same hub.
/// We specifically associate a test with a particular URL, instead of a hub,
/// to be resilient against hub de-registration, so that tests which were routed
/// to the de-registered hub will continue to be routed there, until they are complete
pub type RoutingPrecedentMap = DashMap<String, RoutingDecision>;
pub type Endpoint = Url;

/// A decision made by the routing algorithm, which we associate with a particular Selenium
/// session ID to ensure all requests for that session are sent to the same hub.
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


// For a given request, return a routing decision containing a Hub's endpoint to send that test to.
// If a decision has been previously made for the Selenium session, that will be returned instead.
pub fn make_routing_decision(
    maybe_session_id: Option<String>,
    optional_requested_capabilities: Option<Vec<NewSessionRequestCapability>>,
    routing_map: Arc<RoutingPrecedentMap>,
    state: Arc<HubRouterState>,
) -> Result<RoutingDecision, RoutingError> {

    // If the request has a session ID and we've previously made a routing decision for it,
    // return that previous decision
    if let Some(session_id) = &maybe_session_id {
        if let Some(decision) = routing_map.get(session_id) {
            return Ok(decision.clone());
        }
    }

    // Filter out unhealthy hubs, so that we only consider healthy hubs to send tests to
    let mut healthy_hubs_iter = state
        .hubs
        .iter()
        .filter(|h| h.state.get_readiness() == HubReadiness::Ready)
        .peekable();

    if healthy_hubs_iter.peek().is_none() {
        return Err(RoutingError::NoHealthyNodes(
            "There are no healthy hubs to serve requests to".to_string(),
        ));
    }

    let healthy_hubs: Vec<_> = healthy_hubs_iter.collect();

    // Filter the list of healthy hubs to only those who can satisfy the request,
    // meaning they have a node which can support the requested browser/OS pair
    let (potential_hubs, satisfied_capability): (Option<Vec<&RefMulti<Uuid, Hub>>>, Option<NewSessionRequestCapability>) = match &optional_requested_capabilities {
        None => (Some(healthy_hubs.iter().collect()), None),
        Some(requested_capabilities) => {
            let mut satisfying_hubs: Option<Vec<&RefMulti<Uuid, Hub>>> = None;
            let mut satisfying_capability: Option<NewSessionRequestCapability> = None;
            for capability in requested_capabilities {
                let can_satisfy: Vec<_> = healthy_hubs
                    .iter()
                    .filter(|h| h.can_satisfy_capability(capability))
                    .collect();
                if can_satisfy.len() > 0 {
                    satisfying_hubs = Some(can_satisfy);
                    satisfying_capability = Some(capability.clone());
                    break;
                }
            }
            (satisfying_hubs, satisfying_capability)
        }
    };

    match potential_hubs {
        // If no hubs can satisfy the request, reject the test and return an error
        None => {
            return Err(RoutingError::UnableToSatisfyCapabilities(String::from(
                format!(
                    "No hubs could satisfy capabilities: {:?}",
                    &optional_requested_capabilities
                ),
            )));
        }

        // Otherwise, compute the weights for each hub,
        // and make a weighted random routing decision 
        Some(hubs) => {
            // Compute the weights for each hub.
            // A hub's weight is the number of nodes it has which can run that test,
            // plus the number of these nodes which are empty, so empty nodes count double.
            let keys_and_weights: Vec<_> = hubs
                .iter()
                .map(|h| (h.key(), {
                    let (active, max) = h.state.get_stereotype_fullness(satisfied_capability.clone());
                    u8::max(2 * max - active, 1)
                }))
                .collect();

            // To select a weighted random hub, we compute the total sum of weights,
            // pick a random number from 0 to that weight sum,
            // and skip hubs until the cumulative weight of skipped hubs exceeds
            // the random number.
            let weight_sum = keys_and_weights
                .iter()
                .fold(0, |acc: u64, (_, weight)| acc + *weight as u64);

            let selection_weight_distance: u64 = random::<u64>() % (weight_sum + 1);
            let mut accumulated_weight: u64 = 0;

            let mut selected_hub_uuid: Option<Uuid> = None;
            for (uuid, weight) in keys_and_weights {
                accumulated_weight += weight as u64;
                if accumulated_weight >= selection_weight_distance {
                    selected_hub_uuid = Some(uuid.clone());
                    break;
                }
            }

            if selected_hub_uuid.is_none() {
                warn!("Weighted rolling never selected an endpoint - this shouldn't happen, falling back to uniform random");
                return Err(RoutingError::NoDecision(
                    "Internal Error | Weighted random routing was unable to select a hub".into(),
                ));
            }

            let decision_uuid = match selected_hub_uuid {
                Some(uuid) => uuid,
                None => {
                    return Err(RoutingError::NoDecision(
                        "Internal Error | Unable to select a hub for routing".into(),
                    ));
                }
            };

            let decision_ref = match state.hubs.get(&decision_uuid) {
                Some(hub) => hub,
                None => {
                    return Err(RoutingError::NoDecision(
                        "A hub was selected for routing, but could not be retrieved".into(),
                    ));
                }
            };

            let decision = RoutingDecision::new(
                decision_ref.key().clone(),
                decision_ref.value().meta.url.clone(),
                Instant::now(),
            );

            // Save it if we have a session id
            if let Some(session_id) = maybe_session_id {
                routing_map.insert(session_id.to_string(), decision.clone());
            }
            return Ok(decision);
        }
    }
}


/// Re-write a HTTP request's destination to point at the endpoint
/// in the given routing decision
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
