use std::{collections::HashSet, net::IpAddr, sync::Arc, time::Duration};

use dashmap::DashMap;
use hyper::{Body, Client, Method, Request, Uri};
use tokio::task::JoinSet;

use crate::{
    routing::Endpoint,
    schema::{
        HubStatusJSONSchema, HubStatusNodeJSONSchema, HubStatusNodeSlotIDJSONSchema,
        HubStatusNodeSlotJSONSchema, HubStatusNodeSlotSessionJSONSchema, HubStatusOSInfoJSONSchema,
        HubStatusStereotypeJSONSchema, HubStatusValueJSONSchema, NewSessionRequestCapability,
    },
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HubReadiness {
    Unhealthy,          // not responding to /status requests
    HealthyButNotReady, // responds to /status requests but the ready response is false
    Ready,              // response to /status with ready: true
}

#[derive(Debug, Clone)]
pub struct Hub {
    pub port: u16,
    pub ip: IpAddr,
    pub fullness: u8,
    pub stereotypes: HashSet<HubStatusStereotypeJSONSchema>,
    pub readiness: HubReadiness,
}

impl Hub {
    pub fn new(ip: IpAddr, port: u16) -> Hub {
        Hub {
            ip,
            port,
            fullness: 0,
            stereotypes: HashSet::new(),
            readiness: HubReadiness::Unhealthy,
        }
    }

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
        0
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
    assert_eq!(0, compute_hub_fullness(&t1));
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
