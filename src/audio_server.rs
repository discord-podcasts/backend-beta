use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    ops::Range,
    sync::Arc,
    thread,
};

use actix_web::web::Data;

use crate::Application;

static PORT_RANGE: Range<u16> = 42000..42100;

pub struct AudioServer {
    socket: Arc<UdpSocket>,
    pub port: u16,
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

        thread::spawn(move || {
            let mut buffer = Vec::new();
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

                    let string = String::from_utf8(buffer.clone()).unwrap();
                    // Print the received data and the client's address
                    println!("Received data from {}: {}", src, string);
                }
            }
        });
    }
}
