
use std::net::TcpStream;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};

use crate::routing::{RoutingAlgorithm, EndpointRouter};
use crate::session::TCPSession;


pub fn create_connection_handler(
    connection_reciever: Receiver<TcpStream>,
    session_senders: Vec<Sender<TCPSession>>,
    endpoint_state: Arc<RwLock<EndpointRouter>>,
) -> JoinHandle<()> {
    return thread::spawn(move || {
        let mut round_robin_index = 0;

        for stream in connection_reciever {
            println!("\nHandling a new connection!\n");

            // Configure the recieving sender
            match stream.set_nonblocking(true) {
                Ok(()) => {}
                Err(e) => eprintln!("Could not set source stream to nonblock: {}", e),
            };

            // TODO: Assign the session to an endpoint
            let endpoint_choice = EndpointRouter::route_connection(
                &stream,
                RoutingAlgorithm::RandomUniform,
                endpoint_state.clone(),
            );
            let endpoint_url = endpoint_choice.unwrap();
            let endpoint_connection = TcpStream::connect(endpoint_url);

            // Create and configure the endpoint channel
            match endpoint_connection {
                Ok(endpoint_stream) => {
                    match endpoint_stream.set_nonblocking(true) {
                        Ok(()) => {}
                        Err(e) => eprintln!("Could not set endpoint stream to nonblock: {}", e),
                    };

                    let send_result = session_senders
                        .get(round_robin_index)
                        .unwrap()
                        .send(TCPSession::new(stream, endpoint_stream));

                    round_robin_index += 1;
                    round_robin_index %= session_senders.len();

                    match send_result {
                        Ok(()) => {}
                        Err(send_error) => {
                            eprintln!("Error sending TCP session: {}", send_error)
                        }
                    }
                }
                Err(e) => eprintln!("Error establishing endpoint connection: {}", e),
            };
        }
    });
}
