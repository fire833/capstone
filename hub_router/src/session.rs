use std::io::{self, Read, Write};
use std::net::{TcpStream};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};



pub struct TCPSession {
    source: TcpStream,
    destination: TcpStream,
    dead: bool,
}
impl TCPSession {
    pub fn new(source: TcpStream, destination: TcpStream) -> TCPSession {
        TCPSession {
            source,
            destination,
            dead: false,
        }
    }
}


pub fn create_session_handler(session_reciever: Receiver<TCPSession>) -> JoinHandle<()> {
    return thread::spawn(move || {
        let mut sessions: Vec<TCPSession> = Vec::new();

        let mut read_buffer: [u8; 65536 * 8] = [0; 65536 * 8];

        loop {
            // see if any new sessions have been assigned
            let recv_session_result = session_reciever.try_recv();
            match recv_session_result {
                Ok(new_session) => {
                    sessions.push(new_session);
                }
                Err(recv_error) => match recv_error {
                    mpsc::TryRecvError::Empty => {} // We're happy to continue on if no new sessions are given
                    mpsc::TryRecvError::Disconnected => {} // This result means the session sender (tx) has been disconnected, so we should stop checking for new sessions. TODO.
                },
            }

            fn forward(src: &mut TcpStream, dst: &mut TcpStream, read_buffer: &mut [u8]) -> bool {
                let read_result = src.read(read_buffer);
                match read_result {
                    Ok(num_bytes_read) => {
                        // println!("Read {} bytes:", num_bytes_read);
                        if num_bytes_read == 0 {
                            // println!("I believe the connection has been closed.");
                            return true;
                        }

                        let write_result = dst.write(&read_buffer[0..num_bytes_read]);
                        match write_result {
                            Ok(_num_written) => {} //println!("Wrote {} in response", num_written),
                            Err(e) => eprintln!("Error writing to destination: {}", e),
                        }
                    }
                    Err(ref error) if error.kind() == io::ErrorKind::WouldBlock => {}
                    Err(e) => eprintln!("Error reading from source {}", e),
                }
                return false;
            }

            // Once we've added any new sessions, forward data betwen all session that this thread manages
            for session in &mut sessions {
                let has_source_terminated = forward(
                    &mut session.source,
                    &mut session.destination,
                    &mut read_buffer,
                );
                let has_destination_terminated = forward(
                    &mut session.destination,
                    &mut session.source,
                    &mut read_buffer,
                );
                session.dead = has_source_terminated || has_destination_terminated;
            }

            sessions = sessions.into_iter().filter(|x| !x.dead).collect();
        }
    });
}
