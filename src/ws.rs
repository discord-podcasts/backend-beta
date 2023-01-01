use std::{net::SocketAddr, time::SystemTime};

use actix::{Actor, ActorContext, StreamHandler};
use actix_web::{
    body::BoxBody,
    http::StatusCode,
    web::{Data, Payload, Query},
    HttpRequest, HttpResponse,
};
use actix_web_actors::ws::{self, Message, ProtocolError, WebsocketContext};
use serde::Deserialize;
use tracing::debug;

use crate::{
    authentication::validate_authentication_data,
    events::{Event, EventWrapper, HelloEvent},
    podcast::{PodcastData, PodcastQuery},
    Application,
};

struct PodcastWsSession {
    podcast_id: u32,
    client_id: u32,
    host_client_id: u32,
    app: Data<Application>,
}

#[derive(Deserialize)]
pub struct MessageWrapper {
    pub message_type: u32,
    pub data: serde_json::Value,
}

#[derive(Deserialize)]
struct ClientConnect {
    ip: String,
    port: u32,
}

impl PodcastWsSession {
    fn send_json<T: Event>(&self, ctx: &mut WebsocketContext<Self>, event: &T) {
        let wrapper = EventWrapper {
            event_type: event.event_type(),
            data: event,
        };
        let json = serde_json::to_string(&wrapper);
        match json {
            Ok(json) => ctx.text(json),
            Err(_) => {
                debug!("Failed to serialize an event");
            }
        }
    }

    fn get_podcast(&self) -> PodcastData {
        self.app
            .with_session(self.podcast_id, |session| session.data.clone())
            .unwrap()
    }

    fn get_audio_server_port(&self) -> u16 {
        self.app
            .with_session(self.podcast_id, |session| session.audio_server.port)
            .unwrap()
    }

    fn on_message_receive(&mut self, msg: String) {
        let message = match serde_json::from_str::<MessageWrapper>(msg.as_str()) {
            Ok(message) => message,
            Err(_) => return,
        };

        // A client connected to the udp socket
        if message.message_type == 1 {
            let client_connect = match serde_json::from_value::<ClientConnect>(message.data) {
                Ok(msg) => msg,
                Err(_) => return,
            };

            let address: SocketAddr =
                match format!("{}:{}", client_connect.ip, client_connect.port).parse() {
                    Ok(address) => address,
                    Err(_) => return,
                };

            if self.host_client_id == self.client_id {
                self.app.with_session(self.podcast_id, |session| {
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
                self.app.with_session(self.podcast_id, |session| {
                    session.audio_server.clients.lock().unwrap().insert(address);
                    println!("Client {} is ready to listen", address);
                });
            }
        }
    }
}

impl Actor for PodcastWsSession {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let event = HelloEvent {
            port: self.get_audio_server_port(),
        };
        self.send_json(ctx, &event);
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for PodcastWsSession {
    fn handle(&mut self, msg: Result<ws::Message, ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => return ctx.stop(),
        };

        debug!(?msg);
        match msg {
            Message::Text(text_msg) => {
                println!("This = {}", text_msg.len());
                self.on_message_receive(text_msg.to_string())
            }
            Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

pub async fn websocket(
    Query(query): Query<PodcastQuery>,
    req: HttpRequest,
    stream: Payload,
    app: Data<Application>,
) -> Result<HttpResponse, actix_web::Error> {
    let auth = match validate_authentication_data(&req, &app) {
        Ok(auth) => auth,
        Err(err) => return Err(err),
    };

    let addr = match req.peer_addr() {
        Some(addr) => addr,
        None => {
            debug!("Rejecting websocket connection without peer address");
            return Ok(HttpResponse::new(StatusCode::BAD_REQUEST)
                .set_body(BoxBody::new("Missing peer address")));
        }
    };

    debug!(?addr, "Incoming websocket connection");
    let podcast_data = app.with_session(query.id, |session| session.data.clone());
    match podcast_data {
        Some(podcast_data) => {
            let podcast_ws = PodcastWsSession {
                podcast_id: podcast_data.id,
                client_id: auth.client_id,
                host_client_id: podcast_data.host,
                app,
            };
            ws::start(podcast_ws, &req, stream)
        }
        None => Err(actix_web::error::ErrorNotFound("Podcast doesn't exist")),
    }
}
