use clap::Parser;
use routing::RoutingAlgorithm;
use std::net::{TcpListener, TcpStream, IpAddr};
use std::str::FromStr;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, RwLock};

mod connection;
mod hub_status_schema;
mod routing;
mod session;

use crate::connection::create_connection_handler;
use crate::routing::{Endpoint, EndpointRouter};
use crate::session::{create_session_handler, TCPSession};

#[derive(clap::Parser, Debug)]

struct Args {
    #[arg(long, default_value_t = 6543)]
    bindport: u16,

    #[arg(long, default_value_t = String::from("0.0.0.0"))]
    bindip: String,

    #[arg(long, default_value_t = 4)]
    num_threads: usize,

    #[arg(long, value_enum, default_value_t = RoutingAlgorithm::SourceHash)]
    algorithm: RoutingAlgorithm,

    /// An ip:port endpoint to route to.
    /// Can be used multiple times.
    /// example: --endpoint 192.168.1.1:1234 --endpoint 192.168.1.2:4321
    #[arg(long, value_parser=parse_ip)]
    endpoint: Vec<(String, u16)>
}

fn parse_ip(s: &str) -> Result<(String, u16), String> {
    let mut split = s.split(":");
    let split_ip = split.next();
    let split_port = split.next();

    let ip_parse_result = match split_ip {
        Some(ip_string) => IpAddr::from_str(ip_string),
        None => return Err("Invalid IP address given".to_string())
    };

    let port_parse_result = match split_port {
        Some(port_str) => port_str.parse::<u16>(),
        None => return Err("No port was given".to_string()),
    };


    let ip_addr = match ip_parse_result {
        Ok(addr) => addr,
        Err(e) => return Err(format!("Error parsing endpoint ip: {}", e)),
    };

    let port_u16 = match port_parse_result {
        Ok(u16) => u16,
        Err(e) => return Err(format!("Error parsing endpoint port: {}", e)),
    };

    Ok((ip_addr.to_string(), port_u16))

}

fn main() {
    let args = Args::parse();

    // Bind the TCP server to accept connections
    let bind_string = format!("{}:{}", args.bindip, args.bindport);
    println!("Binding TcpListener at {}", bind_string);
    let listener = TcpListener::bind(bind_string).unwrap();


    // TODO: implement a configuration method to provide endpoints
    let endpoints: Vec<Endpoint> = args.endpoint.iter().map(|(ip, port)| Endpoint::new(port.clone(), ip.clone())).collect();

    if endpoints.len() == 0 {
        panic!("ERROR: No endpoints were specified. Use the --endpoint flag to specify endpoints");
    }

    let endpoint_state = EndpointRouter::new(endpoints);
    let endpoint_state_rw_lock = RwLock::new(endpoint_state);
    let endpoint_state_arc = Arc::new(endpoint_state_rw_lock);
    let (sender, _fetch_thread_handle) =
        EndpointRouter::create_fetch_thread(endpoint_state_arc.clone());
    let _ = sender.send(());

    // Spin up args.num_threads threads to handle TCP sessions as they come in.
    // We send new sessions through the channel to notify a particular thread
    // to handle the given session
    let mut session_senders: Vec<Sender<TCPSession>> = Vec::new();
    let mut session_handler_handles = Vec::new();

    for _id in 0..args.num_threads {
        let (tx, rx): (Sender<TCPSession>, Receiver<TCPSession>) =
            mpsc::channel();
        session_senders.push(tx);
        let session_handler = create_session_handler(rx);
        session_handler_handles.push(session_handler);
    }

    // Spin up a single thread to handle incoming connections.
    // We will assign each connection to a thread, who will manage the tcp session
    let (connection_sender, connection_reciever): (Sender<TcpStream>, Receiver<TcpStream>) =
        mpsc::channel();
    let _connection_handler = create_connection_handler(
        connection_reciever,
        session_senders,
        endpoint_state_arc.clone(),
    );

    // Main, blocking loop. This will never terminate, infinitely handling incoming connections
    for result in listener.incoming() {
        match result {
            Ok(stream) => {
                let _ = connection_sender.send(stream);
            }
            Err(e) => eprintln!("Error accepting incoming TcpStream: {}", e),
        }
    }
}
