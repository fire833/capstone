use std::{net::IpAddr, sync::Arc, time::Duration};

use dashmap::DashMap;
use hyper::{Body, Client, Method, Request, Uri};
use tokio::{task::JoinSet, time::Instant};

use crate::schema::HubStatusJSONSchema;

#[derive(Debug, Copy, Clone)]
pub struct Hub {
    pub port: u16,
    pub ip: IpAddr,
    pub fullness: u8,
    pub last_healthy: Option<Instant>,
}

impl Hub {
    pub fn new(ip: IpAddr, port: u16) -> Hub {
        Hub {
            ip,
            port,
            fullness: 0,
            last_healthy: None,
        }
    }
}

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

#[derive(Debug)]
enum HealthcheckErr {
    DeserializError(serde_json::Error),
    HyperError(hyper::Error),
}

pub async fn hub_healthcheck_thread(hubs: Arc<DashMap<(IpAddr, u16), Hub>>) {
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
            let endpoints: Vec<(IpAddr, u16)> = hubs
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
                            let new_hub = Hub {
                                ip,
                                port,
                                fullness: compute_hub_fullness(&parsed_status),
                                last_healthy: Some(Instant::now()),
                            };
                            hubs.insert((ip, port), new_hub);
                            // println!("Inserted new hub: {:#?}", new_hub);
                        }
                        Err(healthcheck_err) => {
                            eprintln!("Got healthcheck err: {:#?}", healthcheck_err);
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
