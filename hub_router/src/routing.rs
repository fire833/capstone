use std::collections::hash_map::DefaultHasher;
use std::hash::{Hasher};
use std::io::{self};
use std::net::{TcpStream};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Instant, Duration};

use crate::hub_status_schema::HubStatusJSONSchema;


type IP = String;
type Port = u16;


#[derive(Debug)]
pub struct Endpoint {
    port: u16,
    ip: String,
    fullness: u8,
    _last_healthy: Option<Instant>,
}

impl Endpoint {
    pub fn new(port: u16, ip: String) -> Endpoint {
        Endpoint {
            port,
            ip,
            fullness: 0,
            _last_healthy: None,
        }
    }

    fn to_url(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}


#[derive(Debug, Clone, clap::ValueEnum)]
pub enum RoutingAlgorithm {
    SelectFirst,
    RandomUniform,
    RandomWeighted,
    LeastFullFirst,
    SourceHash
}

#[derive(Debug)]
pub struct EndpointRouter {
    endpoints: Vec<Endpoint>,
    last_fetched: Option<Instant>,
    has_pending_refresh_request: bool,
    refresh_request_sender: Option<SyncSender<()>>,
}

impl EndpointRouter {
    pub fn new(endpoints: Vec<Endpoint>) -> EndpointRouter {
        EndpointRouter {
            endpoints: endpoints,
            last_fetched: None,
            has_pending_refresh_request: false,
            refresh_request_sender: None,
        }
    }

    pub fn create_fetch_thread(
        state_rw_lock: Arc<RwLock<EndpointRouter>>,
    ) -> (SyncSender<()>, JoinHandle<()>) {
        let (refresh_request_sender, refresh_request_reciever): (SyncSender<()>, Receiver<()>) =
            mpsc::sync_channel(1024);
        println!("Contending to establish refresh request sender");
        match state_rw_lock.write() {
            Ok(mut state) => {
                state.refresh_request_sender = Some(refresh_request_sender.clone());
            }
            Err(e) => eprintln!("Error obtaining state RwLock to set sender: {}", e),
        }

        let handle = thread::spawn(move || {
            for _ in refresh_request_reciever {

                println!("Handling a refresh request, contending to extract urls");
                
                let endpoint_urls_result: Result<Vec<(IP, Port)>, String> = match state_rw_lock.read()
                {
                    Ok(guard) => Ok(guard.endpoints.iter().map(|e| (e.ip.clone(), e.port)).collect()),
                    Err(e) => Err(format!("RwLock acquisition error: {}", e)),
                };

                match endpoint_urls_result {
                    Ok(endpoint_urls) => {
                        // make the requests
                        let mut new_endpoint_state: Vec<Endpoint> = Vec::new();

                        for (ip, port) in endpoint_urls {
                            let result = ureq::get(&format!("http://{}:{}/status", ip, port)[..]).call();
                            match result {
                                Ok(endpoint_status_response) => {
                                    let json: Result<HubStatusJSONSchema, io::Error> =
                                        endpoint_status_response.into_json();
                                    match json {
                                        Ok(status) => {
                                            if status.value.ready {
                                                println!("Grid is ready!");

                                                let total_available_sessions =
                                                    status.value.nodes.iter().fold(0, |acc, n| {
                                                        acc + (if n.availability.eq("UP") {
                                                            n.maxSessions
                                                        } else {
                                                            0
                                                        })
                                                    });
                                                let total_used_sessions: usize =
                                                    status.value.nodes.iter().fold(0, |acc, n| {
                                                        acc + n.slots
                                                            .iter()
                                                            .filter(|slot| slot.session.is_some())
                                                            .count()
                                                    });
                                                
                                                let endpoint_utilization = if total_available_sessions == 0 {0} else {(total_used_sessions as u32 * 100) / total_available_sessions};

                                                new_endpoint_state.push(Endpoint {
                                                    port,
                                                    ip,
                                                    fullness: endpoint_utilization as u8,
                                                    _last_healthy: Some(Instant::now()),
                                                })
                                            } else {
                                                println!("Grid is not ready.")
                                            }
                                        }
                                        Err(e) => eprintln!("Error parsing json: {}", e),
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error fetching endpoint {}:{} status: {}", ip, port, e)
                                }
                            }
                        }

                        println!("Contending to update endpoint state");
                        match state_rw_lock.write() {
                            Ok(mut state) => {
                                state.endpoints = new_endpoint_state;
                                println!("Updated endpoint state!: {:#?}", state.endpoints);
                                state.last_fetched = Some(Instant::now());
                                state.has_pending_refresh_request = false;
                            }
                            Err(e) => eprintln!(
                                "Unable to open state rw_lock to update after fetch: {}",
                                e
                            )
                        }
                    }
                    Err(e) => eprintln!("Error retrieving list of endpoint urls: {}", e),
                }

                println!("Hopefully dropped write lock?");
            }
        });
        return (refresh_request_sender, handle);
    }

    pub fn route_connection(
        stream: &TcpStream,
        algorithm: RoutingAlgorithm,
        endpoint_router_lock: Arc<RwLock<EndpointRouter>>
    ) -> Result<String, String> {
        let _source_addr = stream.peer_addr();

        {
            println!("Contending for read lock?");
            let read_result = endpoint_router_lock.read();
            let mut should_request_refresh = false;
            let return_value = if let Ok(state) = read_result {
                print!("Should refresh params: {}, {:#?}", state.has_pending_refresh_request, state.last_fetched);
                if !state.has_pending_refresh_request && (state.last_fetched.is_none() || state.last_fetched.unwrap().elapsed().ge(&Duration::from_secs(1))) {
                    print!("Calling out for state re-fresh, conteding for lock");
                    should_request_refresh = true;
                }
                match algorithm {
                    RoutingAlgorithm::SelectFirst => {
                        match state.endpoints.get(0) {
                            Some(endpoint) => Ok(endpoint.to_url()),
                            None => Err(String::from("No available endpoints to route to")),
                        }
                    },
                    RoutingAlgorithm::RandomUniform => {
                        if state.endpoints.is_empty() {
                            Err(String::from("No available endpoints to route to"))
                        } else  {                            
                            println!("Endpoints length: {}", state.endpoints.len());
                            let rand_index: usize = rand::random::<usize>() % state.endpoints.len(); 
                            println!("Routing to rand index {}", rand_index);
                            let endpoint = state.endpoints.get(rand_index).unwrap();
                            Ok(endpoint.to_url())
                        }
                    },
                    RoutingAlgorithm::RandomWeighted => {

                        // The weight is the inverse of fullness.
                        // Something with 0 fullness is given 100 weight,
                        // Something with 100 fullness is given 0 weight.

                        let weight_sum: usize = state.endpoints.iter().fold(0, |acc, el| acc + (100 - el.fullness) as usize);
                        let random_number = rand::random::<usize>() % weight_sum;
                        
                        let mut accumulated_weight: usize = 0;
                        for e in &state.endpoints {
                            accumulated_weight += 100 - e.fullness as usize;
                            if accumulated_weight >= random_number {
                                return Ok(e.to_url())
                            }
                        }
                        return Err("Unable to pick randomly weighted endpoint".to_string());
                    },

                    // This is probably a very bad algorithm and maybe shouldn't be included.
                    // Fullness is computed at an instant in time, which might give inaccurate results,
                    // and is only refreshed once a second.
                    // If a user runs a test suite which all immediately hit the hub router,
                    // they'll all be routed to a single node.
                    // This is generally fine if you're expecting lots of bursts of tests
                    // and want to distribute bursts, but don't care about distributing within a burst.
                    RoutingAlgorithm::LeastFullFirst => {
                        let least_full = state.endpoints.iter().min_by(|e1, e2| e1.fullness.cmp(&e2.fullness));
                        match least_full {
                            Some(endp) => Ok(endp.to_url()),
                            None => Err("To endpoint was available to satisfy least full first".to_string()),
                        }
                    },
                    RoutingAlgorithm::SourceHash => {
                        let mut hasher = DefaultHasher::new();
                        match stream.peer_addr() {
                            Ok(addr) => {
                                match addr.ip() {
                                    std::net::IpAddr::V4(v4) => hasher.write(&v4.octets()),
                                    std::net::IpAddr::V6(v6) => hasher.write(&v6.octets()),
                                }
                                hasher.write(&addr.port().to_le_bytes());
                                let hash = hasher.finish();
                                let index = (hash as usize) % state.endpoints.len();
                                let ep = state.endpoints.get(index);
                                match ep {
                                    Some(endpoint) => Ok(endpoint.to_url()),
                                    None => Err(String::from("No endpoints were available")),
                                }
                            },
                            Err(_) => Err(String::from("Could not get stream peer addr in SourceHash")),
                        }
                    },
                }
            } else {    
                Err(read_result.unwrap_err().to_string())
            };

            if should_request_refresh {
                match endpoint_router_lock.write() {
                    Ok(mut writeable_state) => {
                        println!("Acquired write lock!");
                        writeable_state.has_pending_refresh_request = true;
                        if let Some(refresh_request_sender) = &writeable_state.refresh_request_sender {
                            let _ = refresh_request_sender.send(());
                        } else {
                            writeable_state.has_pending_refresh_request = false;
                        }
                    }
                    Err(_) => todo!(),
                }
            }

            return return_value;
        }

    }
}
