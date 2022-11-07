use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    ops::Range,
};

use actix_web::web::Data;

use crate::Application;

static PORT_RANGE: Range<u16> = 42000..42100;

pub struct AudioServer {
    port: u16,
}

impl AudioServer {
    pub fn create(app: &Data<Application>) -> Option<UdpSocket> {
        let used_ports = app
            .sessions
            .lock()
            .unwrap()
            .values()
            .map(|podcast| podcast.audio_server.local_addr())
            .filter(|address| address.is_ok())
            .map(|address| address.unwrap().port())
            .collect::<Vec<u16>>();
        let free_ports = PORT_RANGE
            .clone()
            .filter(|port| !used_ports.contains(port))
            .collect::<Vec<u16>>();

        for port in free_ports {
            let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
            let server = UdpSocket::bind(address);
            match server {
                Ok(server) => return Some(server),
                Err(_) => continue,
            }
        }

        return None;
    }
}
