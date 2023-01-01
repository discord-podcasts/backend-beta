use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    ops::Range,
    sync::{Arc, Mutex},
    thread,
};

use actix_web::web::Data;

use crate::Application;

static PORT_RANGE: Range<u16> = 42000..42100;

pub struct AudioServer {
    socket: Arc<UdpSocket>,
    pub port: u16,
    pub host_address: Option<SocketAddr>, // UDP socket address of the host
    pub clients: Arc<Mutex<HashSet<SocketAddr>>>, // UDP socket addresses of the clients
}

impl AudioServer {
    pub fn create(app: &Data<Application>) -> Option<AudioServer> {
        let used_ports = app.sessions(|sessions| {
            sessions
                .values()
                .map(|podcast| podcast.audio_server.socket.local_addr())
                .filter(|address| address.is_ok())
                .map(|address| address.unwrap().port())
                .collect::<Vec<u16>>()
        });
        let free_ports = PORT_RANGE
            .clone()
            .filter(|port| !used_ports.contains(port))
            .collect::<Vec<u16>>();

        for port in free_ports {
            let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
            let server = UdpSocket::bind(address);
            match server {
                Ok(socket) => {
                    let port = socket.local_addr().unwrap().port();
                    println!("Created audio server at 127.0.0.1:{}", port);

                    let audio_server = AudioServer {
                        socket: Arc::new(socket),
                        port,
                        host_address: None,
                        clients: Arc::from(Mutex::from(HashSet::new())),
                    };

                    return Some(audio_server);
                }
                Err(_) => continue,
            }
        }

        None
    }

    pub fn listen(&self, host_address: SocketAddr) {
        let socket = self.socket.clone();
        let thread_safe_clients = self.clients.clone();

        thread::spawn(move || {
            let mut buffer = Vec::new();
            let clients = thread_safe_clients;
            loop {
                // Reset buffer
                buffer.clear();
                buffer.resize(2000, 0);

                if let Ok((size, src)) = socket.recv_from(buffer.as_mut_slice()) {
                    buffer.resize(size, 0);

                    if src != host_address {
                        println!("WANT = {} | HAVE = {}", host_address, src);

                        continue;
                    }

                    let array = buffer.as_slice();
                    let clients = clients.lock().unwrap();
                    for client in clients.iter() {
                        let result = socket.send_to(array, client);
                        if result.is_err() {
                            println!("Failed to send")
                        }
                    }
                    // Print the received data and the client's address
                    //println!("Received data from {}: {}", src, string);
                }
            }
        });
    }
}
