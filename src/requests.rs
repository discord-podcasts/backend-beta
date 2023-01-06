use serde::Deserialize;
use serde_json::Value;
use std::net::SocketAddr;
use std::time::SystemTime;

use crate::websocket::ws::PodcastWsSession;

#[derive(Deserialize)]
pub struct MessageWrapper {
    pub message_type: u32,
    pub data: Value,
}

pub trait Request {
    fn request_type() -> u32;
    fn handle(&self, session: &PodcastWsSession);
}

/// Received when a client connected to the audio socket.
#[derive(Deserialize)]
pub struct ClientConnect {
    ip: String,
    port: u32,
}

impl Request for ClientConnect {
    fn request_type() -> u32 {
        1
    }

    fn handle(&self, session: &PodcastWsSession) {
        let address: SocketAddr = match format!("{}:{}", self.ip, self.port).parse() {
            Ok(addr) => addr,
            Err(_) => return,
        };

        if session.host_client_id == session.client_id {
            session.app.with_session(session.podcast_id, |session| {
                let current_time_millis = SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis();

                session.data.active_since = Some(current_time_millis);
                session.audio_server.host_address = Some(address);
                println!("Host {} is ready to send", address);

                session.audio_server.listen(address);
            });
        } else {
            session.app.with_session(session.podcast_id, |session| {
                session.audio_server.clients.lock().unwrap().insert(address);
                println!("Client {} is ready to listen", address);
            });
        }
    }
}
