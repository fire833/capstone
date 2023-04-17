use std::{
    collections::{hash_map::DefaultHasher, HashSet},
    hash::{Hash, Hasher},
    sync::Arc,
    time::Duration,
};

use base64::Engine;
use hyper::{Body, Client, Method, Request};
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use crate::{
    routing::Endpoint,
    schema::{
        HubStatusJSONSchema, HubStatusNodeJSONSchema, HubStatusNodeSlotIDJSONSchema,
        HubStatusNodeSlotJSONSchema, HubStatusNodeSlotSessionJSONSchema, HubStatusOSInfoJSONSchema,
        HubStatusStereotypeJSONSchema, HubStatusValueJSONSchema, NewSessionRequestCapability,
    },
    state::HubRouterState,
};
use log::{info, warn};
use url::Url;
use uuid::Uuid;

/// HubReadiness represents the current status of a Hub as a enum.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum HubReadiness {
    /// not responding to /status requests
    Unhealthy,

    /// response to /status with ready: true
    Ready,
}

impl Default for HubReadiness {
    fn default() -> Self {
        Self::Unhealthy
    }
}

/// Hub is an internal representation for a remote Selenium Hub instance
/// we wish to forward tests to. This will be serialized to configuration files.
/// To view runtime statistics and information about a Hub, this type can be cast
/// to a HubExternal, which will serialize with all information for API consumption.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Hub {
    /// meta is the metadata associated with a hub. This includes the
    /// endpoint, uuid, name, etc.
    pub meta: HubMetadata,

    /// state is the transient runtime state associated with a running hub.
    /// This includes fullness, capabilities, etc.
    pub state: HubState,
}

/// Persistent metadata ssociated with a hub.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HubMetadata {
    pub name: String,

    #[serde(serialize_with = "crate::utils::serialize_url")]
    #[serde(deserialize_with = "crate::utils::deserialize_url")]
    pub url: url::Url,

    #[serde(serialize_with = "crate::utils::serialize_uuid")]
    #[serde(deserialize_with = "crate::utils::deserialize_uuid")]
    pub uuid: uuid::Uuid,
}

/// Transient state associated with a hub at runtime.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HubState {
    pub fullness: u8,
    pub stereotypes: HashSet<HubStatusStereotypeJSONSchema>,
    pub readiness: HubReadiness,
    pub consecutive_healthcheck_failures: u8,
}

impl HubState {
    pub fn get_fullness(&self) -> u8 {
        self.fullness
    }

    pub fn get_readiness(&self) -> HubReadiness {
        self.readiness
    }
}

impl Default for HubState {
    fn default() -> Self {
        HubState {
            fullness: 0,
            stereotypes: HashSet::new(),
            readiness: HubReadiness::Unhealthy,
            consecutive_healthcheck_failures: 0,
        }
    }
}

impl Hub {
    pub fn new(url: Url) -> Self {
        let mut hasher = DefaultHasher::new();
        (url).hash(&mut hasher);
        let hash = hasher.finish() as u32;
        let base64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash.to_le_bytes());
        Hub::new_with_name(&format!("Hub_{}", base64), url)
    }

    pub fn from_meta(meta: HubMetadata) -> Self {
        Self {
            meta,
            state: HubState::default(),
        }
    }

    // Create a new Hub instance with a predefined name.
    pub fn new_with_name(name: &str, url: Url) -> Self {
        Self {
            meta: HubMetadata {
                name: name.into(),
                url,
                uuid: uuid::Uuid::new_v4(),
            },
            state: HubState {
                fullness: 0,
                stereotypes: HashSet::new(),
                readiness: HubReadiness::Unhealthy,
                consecutive_healthcheck_failures: 0,
            },
        }
    }

    /// Check to make sure that the current Hub will support the desired capability.
    pub fn can_satisfy_capability(&self, capability: &NewSessionRequestCapability) -> bool {
        self.state.stereotypes.iter().any(|stereotype| {
            let satisfies_browser = capability.browserName.is_none()
                || (&stereotype.browserName)
                    .eq_ignore_ascii_case(capability.browserName.as_ref().unwrap());

            let satisfies_platform_name = capability.platformName.is_none()
                || (&stereotype.platformName)
                    .eq_ignore_ascii_case(capability.platformName.as_ref().unwrap());

            satisfies_browser && satisfies_platform_name
        })
    }

    pub fn clone_from_meta(&self) -> Self {
        Self {
            meta: self.meta.clone(),
            state: HubState::default(),
        }
    }

    /// Check to make sure a Hub is both ready and available, and satisfies the
    /// desired capability.
    #[allow(unused)]
    pub fn is_ready_and_capable(&self, capability: &NewSessionRequestCapability) -> bool {
        self.can_satisfy_capability(capability) && self.state.readiness == HubReadiness::Ready
    }

    pub fn fail_healthcheck(&mut self) -> HubReadiness {
        if self.state.consecutive_healthcheck_failures < u8::MAX {
            self.state.consecutive_healthcheck_failures += 1;
        }
        if self.state.consecutive_healthcheck_failures >= 3 {
            self.state.readiness = HubReadiness::Unhealthy;
        }
        return self.state.readiness;
    }

    pub fn succeed_healthcheck(&mut self) -> HubReadiness {
        self.state.consecutive_healthcheck_failures = 0;
        self.state.readiness = HubReadiness::Ready;
        return self.state.readiness;
    }
}

/// Primary function to calculate the percentage of fullness of a hub based on its
/// returned status API schema. Returns a value between 0 and 100.
pub fn compute_hub_fullness(status: &HubStatusJSONSchema) -> u8 {
    let max_slots = status
        .value
        .nodes
        .iter()
        .fold(0, |acc, node| acc + node.maxSessions);

    let running_slots = status.value.nodes.iter().fold(0, |acc, node| {
        acc + node.slots.iter().fold(0, |slot_acc, slot| {
            slot_acc + (if slot.session.is_some() { 1 } else { 0 })
        })
    });

    if max_slots == 0 {
        100
    } else {
        ((running_slots * 100) / max_slots) as u8
    }
}

/// Mock function for testing to create a new HubStatusJSONSchema with
/// the specified number of nodes, number of maxSessions on each node,
/// and number of running sessions per node.
#[allow(unused)]
fn mock_status_schema(max_sessions: u32, num_nodes: u32, num_running: u32) -> HubStatusJSONSchema {
    let mut mock = HubStatusJSONSchema {
        value: HubStatusValueJSONSchema {
            ready: true,
            message: String::from("UP"),
            nodes: vec![],
        },
    };

    for _ in 0..num_nodes {
        let mut node = HubStatusNodeJSONSchema {
            id: String::from(""),
            uri: String::from(""),
            maxSessions: max_sessions,
            slots: vec![],
            availability: String::from("UP"),
            version: String::from("undefined"),
            osInfo: HubStatusOSInfoJSONSchema {
                arch: String::from("x86_64"),
                name: String::from("foo bar"),
                version: String::from("undefined"),
            },
        };

        for _ in 0..num_running {
            let session = HubStatusNodeSlotJSONSchema {
                lastStarted: String::from("null"),
                id: HubStatusNodeSlotIDJSONSchema {
                    hostId: String::from("undefined"),
                    id: String::from("undefined"),
                },
                session: Some(HubStatusNodeSlotSessionJSONSchema {
                    capabilities: None,
                    sessionId: String::from("undefined"),
                    start: String::from("undefined"),
                    uri: String::from(""),
                    stereotype: HubStatusStereotypeJSONSchema {
                        browserName: String::from("nil"),
                        platformName: String::from("nil"),
                    },
                }),
                stereotype: HubStatusStereotypeJSONSchema {
                    browserName: String::from("nil"),
                    platformName: String::from("nil"),
                },
            };

            node.slots.push(session);
        }

        mock.value.nodes.push(node);
    }

    mock
}

/// Silly little test to verify the compute_hub_fullness method above.
#[test]
fn compute_hub_fullness_test() {
    let t1 = HubStatusJSONSchema {
        value: HubStatusValueJSONSchema {
            ready: true,
            message: String::from("UP"),
            nodes: vec![],
        },
    };

    let t2 = mock_status_schema(10, 15, 2);
    let t3 = mock_status_schema(10, 10, 6);
    let t4 = mock_status_schema(10, 10, 9);
    assert_eq!(100, compute_hub_fullness(&t1));
    assert_eq!(20, compute_hub_fullness(&t2));
    assert_eq!(60, compute_hub_fullness(&t3));
    assert_eq!(90, compute_hub_fullness(&t4));
}

#[derive(Debug)]
enum HealthcheckErr {
    DeserializError(serde_json::Error),
    HyperError(hyper::Error),
}

pub async fn hub_healthcheck_thread(state: Arc<HubRouterState>) {
    info!("starting healthcheck thread");
    let mut healthcheck_interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        let mut request_futures: JoinSet<(Uuid, Result<HubStatusJSONSchema, HealthcheckErr>)> = {
            let mut join_set: JoinSet<(Uuid, Result<HubStatusJSONSchema, HealthcheckErr>)> =
                JoinSet::new();
            let endpoints: Vec<(Uuid, Endpoint)> = state
                .hubs
                .iter()
                .map(|h| (h.meta.uuid, h.meta.url.clone()))
                .collect();

            for (hub_uuid, _url) in endpoints {
                let client = Client::new();
                let mut request_url = _url.clone();
                request_url.set_path("/status");

                let request: Request<Body> = Request::builder()
                    .uri(request_url.to_string())
                    .method(Method::GET)
                    .body(hyper::body::Body::empty())
                    .unwrap();

                join_set.spawn(async move {
                    let response_result = client.request(request).await;
                    match response_result {
                        Ok(response) => {
                            let (_parts, body) = response.into_parts();
                            let body_bytes = hyper::body::to_bytes(body).await.unwrap();
                            let parsed_struct_result: Result<
                                HubStatusJSONSchema,
                                serde_json::Error,
                            > = serde_json::from_slice(&body_bytes);
                            match parsed_struct_result {
                                Ok(parsed) => (hub_uuid, Ok(parsed)),
                                Err(e) => (hub_uuid, Err(HealthcheckErr::DeserializError(e))),
                            }
                        }
                        Err(err) => (hub_uuid, Err(HealthcheckErr::HyperError(err))),
                    }
                });
            }
            join_set
        };

        while let Some(res) = request_futures.join_next().await {
            match res {
                Ok((url, status_result)) => match status_result {
                    Ok(parsed_status) => {
                        let is_ready = parsed_status.value.nodes.len() > 0;
                        match state.hubs.get_mut(&url) {
                            Some(mut hub) => {
                                hub.state.fullness = compute_hub_fullness(&parsed_status);
                                hub.state.readiness = if is_ready {
                                    hub.succeed_healthcheck()
                                } else {
                                    hub.fail_healthcheck()
                                };
                                parsed_status.value.nodes.iter().for_each(|node| {
                                    node.slots.iter().for_each(|slot| {
                                        if !hub.state.stereotypes.contains(&slot.stereotype) {
                                            hub.state.stereotypes.insert(slot.stereotype.clone());
                                        }
                                    })
                                });
                            }
                            None => {
                                warn!("Somehow, a hub which we performed a healthcheck for is not in the map?");
                            }
                        }
                    }
                    Err(healthcheck_err) => {
                        eprintln!("Got healthcheck err: {:#?}", healthcheck_err);
                        match state.hubs.get_mut(&url) {
                            Some(mut hub) => {
                                hub.fail_healthcheck();
                            }
                            None => {
                                warn!("Somehow, a hub which we performed a healthcheck for is not in the map?")
                            }
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Healthcheck task failed to complete: {}", e)
                }
            }
        }

        healthcheck_interval.tick().await;
    }
}
