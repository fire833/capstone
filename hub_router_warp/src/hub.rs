//! The internal representation for a Selenium hub which
//! is registered with the Hub Router


use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::{Hash, Hasher},
    sync::Arc,
    time::Duration,
};

use base64::Engine;
use hyper::{Body, Client, Method, Request};
use serde::{Deserialize, Serialize};
use tokio::{task::JoinSet, time::timeout};
use utoipa::ToSchema;

use crate::{
    routing::Endpoint,
    schema::{
        HubStatusJSONSchema, HubStatusNodeJSONSchema, HubStatusNodeSlotIDJSONSchema,
        HubStatusNodeSlotJSONSchema, HubStatusNodeSlotSessionJSONSchema, HubStatusOSInfoJSONSchema,
        HubStatusStereotypeJSONSchema, HubStatusValueJSONSchema, NewSessionRequestCapability,
    },
    state::{HubRouterPrimitiveConfigs, HubRouterState},
};
use log::{info, warn};
use url::Url;
use uuid::Uuid;

/// HubReadiness represents the current status of a Hub as a enum.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum HubReadiness {
    /// not responding to /status requests, or responds to /status but has no nodes
    Unhealthy,

    /// response to /status with at least a single healthy node
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct Hub {
    /// meta is the metadata associated with a hub. This includes the
    /// endpoint, uuid, name, etc.
    pub meta: HubMetadata,

    /// state is the transient runtime state associated with a running hub.
    /// This includes fullness, capabilities, etc.
    pub state: HubState,
}

/// Persistent metadata ssociated with a hub.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct HubState {
    #[serde(skip)] // Skip for now, serde doesn't like a struct being the key
    pub fullness: HashMap<NewSessionRequestCapability, (u8, u8)>,
    pub stereotypes: HashSet<HubStatusStereotypeJSONSchema>,
    pub readiness: HubReadiness,
    pub consecutive_healthcheck_failures: u8,
}

impl HubState {

    /// Compute a tuple of (currently running sessions, maximum capacity)
    /// for a particular browser/OS request.
    /// Dividing these numbers gives the percent fullness for running 
    /// a particular browser/OS test on this hub.
    pub fn get_stereotype_fullness(
        &self,
        maybe_capability: Option<NewSessionRequestCapability>,
    ) -> (u8, u8) {
        let (mut active_sessions, mut max_sessions) = (0, 0);
        let capability = match maybe_capability {
            Some(c) => c,
            None => NewSessionRequestCapability {
                browserName: None,
                platformName: None,
            },
        };
        for (hub_capability, (active, max)) in &self.fullness {
            if capability.satisfied_by(hub_capability) {
                active_sessions += active;
                max_sessions += max;
            }
        }
        (active_sessions, max_sessions)
    }

    pub fn get_readiness(&self) -> HubReadiness {
        self.readiness
    }
}

impl Default for HubState {
    fn default() -> Self {
        HubState {
            fullness: HashMap::new(),
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
            state: HubState::default(),
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

    /// Called when a hub fails a healthcheck.
    /// If a hub fails 3 consecutive healthchecks, it will be marked unhealthy
    pub fn fail_healthcheck(&mut self) -> HubReadiness {
        if self.state.consecutive_healthcheck_failures < u8::MAX {
            self.state.consecutive_healthcheck_failures += 1;
        }
        if self.state.consecutive_healthcheck_failures >= 3 {
            self.state.readiness = HubReadiness::Unhealthy;
        }
        return self.state.readiness;
    }

    /// Called when a hub succeeds a healthcheck.
    /// As soon as a hub suceeds a healthcheck, it will be considered healthy.
    pub fn succeed_healthcheck(&mut self) -> HubReadiness {
        self.state.consecutive_healthcheck_failures = 0;
        self.state.readiness = HubReadiness::Ready;
        return self.state.readiness;
    }
}

/// Primary function to calculate the percentage of fullness of a hub based on its
/// returned status API schema. Returns a map from capabilities (browser/OS pairs) to a
/// tuple of (running sessions, session capacity).
pub fn compute_hub_fullness(
    status: &HubStatusJSONSchema,
) -> HashMap<NewSessionRequestCapability, (u8, u8)> {
    let mut map: HashMap<NewSessionRequestCapability, (u8, u8)> = HashMap::new();

    for node in &status.value.nodes {
        for slot in &node.slots {
            let key: NewSessionRequestCapability = slot.stereotype.clone().into();
            if !map.contains_key(&key) {
                map.insert(key.clone(), (0, 0));
            }
            match map.get(&key) {
                Some((active_slots, total_slots)) => {
                    let active = if slot.session.is_some() { 1 } else { 0 };
                    map.insert(key, (active_slots + active, total_slots + 1));
                }
                None => {
                    warn!("Map was corrupted - slot which we just inserted is not available");
                }
            }
        }
    }
    map
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


/// Represents all of the reasons a hub could fail a healthcheck, with an associated
/// error message which generated that type of failure
#[derive(Debug)]
enum HealthcheckErr {
    DeserializError(serde_json::Error),
    HyperError(hyper::Error),
    Timeout(String),
}


/// The long-running thread which polls hubs for their healthiness and fullness.
pub async fn hub_healthcheck_thread(state: Arc<HubRouterState>) {
    info!("starting healthcheck thread");

    // Extract the healthcheck interval from config, and turn it into a tokio interval
    let mut healthcheck_interval = match state.configs.read() {
        Ok(conf) => {
            tokio::time::interval(Duration::from_secs(conf.healthcheck_thread_interval))
        },
        Err(e) => {
            warn!("Unable to acquire read lock for configs: {} - defaulting to 1 second healthcheck interval", e);
            tokio::time::interval(Duration::from_secs(1))
        },
    };

    loop {

        // Make a request to /status on each hub
        let mut request_futures: JoinSet<(Uuid, Result<HubStatusJSONSchema, HealthcheckErr>)> = {
            let mut join_set: JoinSet<(Uuid, Result<HubStatusJSONSchema, HealthcheckErr>)> =
                JoinSet::new();
            let endpoints: Vec<(Uuid, Endpoint)> = state
                .clone()
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

                let state_clone = state.clone();
                join_set.spawn(async move {
                    let interval = match state_clone.configs.read() {
                        Ok(conf) => conf.healthcheck_timeout,
                        Err(e) => {
                            warn!("RwLock was poisoned getting healthcheck timeout: {}", e);
                            HubRouterPrimitiveConfigs::default().healthcheck_thread_interval
                        }
                    };
                    let response_result_with_timeout =
                        timeout(Duration::from_secs(interval), client.request(request)).await;
                    match response_result_with_timeout {
                        Ok(response_result) => match response_result {
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
                        },
                        Err(_elapsed) => (
                            hub_uuid,
                            Err(HealthcheckErr::Timeout(format!(
                                "Healthcheck to {} timed out",
                                _url
                            ))),
                        ),
                    }
                });
            }
            join_set
        };

        // For each response to /status (or timeout), inspect the request to determine if the hub is healthy
        // and if so, update its fullness metrics
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
                        warn!("Got healthcheck err: {:#?}", healthcheck_err);
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
                    warn!("Healthcheck task failed to complete: {}", e)
                }
            }
        }

        healthcheck_interval.tick().await;
    }
}
