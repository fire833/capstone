use std::{collections::HashSet, default, net::IpAddr, sync::Arc, time::Duration};

use dashmap::DashMap;
use hyper::{Body, Client, Method, Request, Uri};
use serde::{de::Visitor, Deserialize, Serialize};
use tokio::task::JoinSet;
use url::Url;

use crate::{
    routing::Endpoint,
    schema::{
        HubStatusJSONSchema, HubStatusNodeJSONSchema, HubStatusNodeSlotIDJSONSchema,
        HubStatusNodeSlotJSONSchema, HubStatusNodeSlotSessionJSONSchema, HubStatusOSInfoJSONSchema,
        HubStatusStereotypeJSONSchema, HubStatusValueJSONSchema, NewSessionRequestCapability,
    },
};

/// SerializableURL is a wrapper around the standard url::Url type
/// so that we can implement serialize and deserialize for it.
/// We can't implement traits for aliased types that are external.
///
/// Pretty stupid, but its how rust stays happy.
#[derive(Clone, Debug)]
pub struct SerializableURL {
    pub url: url::Url,
}

struct UrlVisitor;

impl<'de> Visitor<'de> for UrlVisitor {
    type Value = SerializableURL;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid, parseable url string")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match Url::parse(&v) {
            Ok(url) => Ok(SerializableURL { url }),
            Err(e) => Err(E::custom(e.to_string())),
        }
    }
}

impl Serialize for SerializableURL {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.url.to_string())
    }
}

impl<'de> Deserialize<'de> for SerializableURL {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(UrlVisitor)
    }
}

impl Default for SerializableURL {
    fn default() -> Self {
        Self {
            url: Url::parse("http://localhost:4444").expect("this url should be valid"),
        }
    }
}

/// HubReadiness represents the current status of a Hub as a enum.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum HubReadiness {
    /// not responding to /status requests
    Unhealthy,

    /// responds to /status requests but the ready response is false
    HealthyButNotReady,

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hub {
    #[serde(alias = "name")]
    pub name: String,

    // The URL endpoint to reach this Hub.
    #[serde(alias = "url")]
    url_str: String,

    #[serde(skip)]
    url: SerializableURL,

    #[serde(skip)]
    fullness: u8,
    #[serde(skip)]
    stereotypes: HashSet<HubStatusStereotypeJSONSchema>,
    #[serde(skip)]
    readiness: HubReadiness,
}

impl Default for Hub {
    fn default() -> Self {
        let url_str = "http://localhost:4444";

        let url = SerializableURL {
            url: Url::parse(url_str).expect("this url should be valid"),
        };

        Self {
            name: String::from("unknown"),
            url_str: String::from(url_str),
            url,
            fullness: 0,
            stereotypes: HashSet::new(),
            readiness: HubReadiness::Unhealthy,
        }
    }
}

impl Hub {
    /// Initialize a new Hub instance.
    pub fn new(url: &str) -> Result<Hub, url::ParseError> {
        match url::Url::parse(url) {
            Ok(url_parsed) => {
                return Ok(Hub {
                    name: String::from("unknown"),
                    url_str: String::from(url),
                    url: SerializableURL { url: url_parsed },
                    fullness: 0,
                    stereotypes: HashSet::new(),
                    readiness: HubReadiness::Unhealthy,
                })
            }
            Err(e) => return Err(e),
        }
    }

    // Create a new Hub instance with a predefined name.
    pub fn new_with_name(name: &str, url: &str) -> Result<Hub, url::ParseError> {
        match url::Url::parse(url) {
            Ok(url_parsed) => {
                return Ok(Hub {
                    name: String::from(name),
                    url_str: String::from(url),
                    url: SerializableURL { url: url_parsed },
                    fullness: 0,
                    stereotypes: HashSet::new(),
                    readiness: HubReadiness::Unhealthy,
                })
            }
            Err(e) => return Err(e),
        }
    }

    /// Check to make sure that the current Hub will support the desired capability.
    pub fn can_satisfy_capability(&self, capability: &NewSessionRequestCapability) -> bool {
        self.stereotypes.iter().any(|stereotype| {
            let satisfies_browser = capability.browserName.is_none()
                || (&stereotype.browserName)
                    .eq_ignore_ascii_case(capability.browserName.as_ref().unwrap());

            let satisfies_platform_name = capability.platformName.is_none()
                || (&stereotype.platformName)
                    .eq_ignore_ascii_case(capability.platformName.as_ref().unwrap());

            satisfies_browser && satisfies_platform_name
        })
    }

    /// Check to make sure a Hub is both ready and available, and satisfies the
    /// desired capability.
    #[allow(unused)]
    pub fn is_ready_and_capable(&self, capability: &NewSessionRequestCapability) -> bool {
        self.can_satisfy_capability(capability) && self.readiness == HubReadiness::Ready
    }
}

/// HubExternal is the object for serializing/deserializing internal Hub information
/// for reading and writing via the external API.
#[derive(Debug)]
#[allow(non_snake_case, unused)]
pub struct HubExternal<'a> {
    hub: &'a Hub,
}

impl<'a> Serialize for HubExternal<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.hub.name)?;
        serializer.serialize_str(&self.hub.url.url.to_string())?;
        serializer.serialize_i8(self.hub.fullness as i8)?;
        match self.hub.readiness {
            HubReadiness::Ready => serializer.serialize_str("ready"),
            HubReadiness::Unhealthy => serializer.serialize_str("unhealthy"),
            HubReadiness::HealthyButNotReady => serializer.serialize_str("healthynotready"),
        }
    }
}

impl<'de, 'a> Deserialize<'de> for HubExternal<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
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

pub async fn hub_healthcheck_thread(hubs: Arc<DashMap<Endpoint, Hub>>) {
    println!("called hub healthcheck thread");
    let mut healthcheck_interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        let mut request_futures: JoinSet<(
            IpAddr,
            u16,
            Result<HubStatusJSONSchema, HealthcheckErr>,
        )> = {
            let mut join_set: JoinSet<(IpAddr, u16, Result<HubStatusJSONSchema, HealthcheckErr>)> =
                JoinSet::new();
            let endpoints: Vec<Endpoint> = hubs
                .iter()
                .map(|h| (h.ip.clone(), h.port.clone()))
                .collect();

            for (ip, port) in endpoints {
                let client = Client::new();

                let request: Request<Body> = Request::builder()
                    .uri(
                        Uri::builder()
                            .scheme("http")
                            .authority(format!("{}:{}", ip, port))
                            .path_and_query("/status")
                            .build()
                            .unwrap(),
                    )
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
                                Ok(parsed) => (ip, port, Ok(parsed)),
                                Err(e) => (ip, port, Err(HealthcheckErr::DeserializError(e))),
                            }
                        }
                        Err(err) => (ip, port, Err(HealthcheckErr::HyperError(err))),
                    }
                });
            }
            join_set
        };

        while let Some(res) = request_futures.join_next().await {
            match res {
                Ok((ip, port, status_result)) => {
                    match status_result {
                        Ok(parsed_status) => {
                            let is_ready = parsed_status.value.ready;
                            match hubs.get_mut(&(ip, port)) {
                                Some(mut hub) => {
                                    hub.fullness = compute_hub_fullness(&parsed_status);
                                    hub.readiness = if is_ready {
                                        HubReadiness::Ready
                                    } else {
                                        HubReadiness::HealthyButNotReady
                                    };
                                    parsed_status.value.nodes.iter().for_each(|node| {
                                        node.slots.iter().for_each(|slot| {
                                            if !hub.stereotypes.contains(&slot.stereotype) {
                                                hub.stereotypes.insert(slot.stereotype.clone());
                                            }
                                        })
                                    });
                                    // println!("Updated hub: {:#?}", hub.value());
                                }
                                None => {
                                    eprintln!("[WARN] Somehow, a hub which we performed a healthcheck for is not in the map?")
                                }
                            }
                        }
                        Err(healthcheck_err) => {
                            eprintln!("Got healthcheck err: {:#?}", healthcheck_err);
                            match hubs.get_mut(&(ip, port)) {
                                Some(mut hub) => {
                                    hub.readiness = HubReadiness::Unhealthy;
                                }
                                None => {
                                    eprintln!("[WARN] Somehow, a hub which we performed a healthcheck for is not in the map?")
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Healthcheck task failed to complete: {}", e)
                }
            }
        }

        healthcheck_interval.tick().await;
    }
}
