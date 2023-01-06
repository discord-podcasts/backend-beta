use crate::{
    authentication::validate_authentication_data,
    podcast::{Podcast, PodcastQuery},
    websocket::events::close_connection_event,
    Application,
};
use crate::{
    requests::{ClientConnect, MessageWrapper, Request},
    websocket::events::close_connection_event::CloseConnectionEvent,
};
use actix::{Actor, ActorContext, Addr, AsyncContext, StreamHandler};
use actix_web::{
    body::BoxBody,
    http::StatusCode,
    test::TestBuffer,
    web::{Data, Payload, Query},
    HttpRequest, HttpResponse,
};
use actix_web_actors::ws::{self, ProtocolError, WebsocketContext};
use tracing::debug;

use super::events::{events::Event, hello_event::HelloEvent};

#[derive(Clone)]
pub struct PodcastWsSession {
    pub addr: Option<Addr<PodcastWsSession>>,
    pub(crate) podcast_id: u32,
    pub(crate) client_id: u32,
    pub(crate) host_client_id: u32,
    pub(crate) app: Data<Application>,
}

impl PodcastWsSession {
    pub fn send_json<T: Event + actix::Message>(&self, event: &T) {
        if let Some(addr) = &self.addr {
            println!("Sending to {}", self.client_id);
            let t = event.clone();
            addr.do_send();
        }
    }

    fn is_host(&self) -> bool {
        return self.client_id == self.host_client_id;
    }

    fn with_podcast<F>(&self, f: F)
    where
        F: FnOnce(&mut Podcast) -> (),
    {
        self.app.with_session(self.podcast_id, |session| f(session));
    }

    fn on_message_receive(&mut self, msg: String) {
        let message = match serde_json::from_str::<MessageWrapper>(msg.as_str()) {
            Ok(message) => message,
            Err(_) => return,
        };

        // A client connected to the udp socket
        if message.message_type == 1 {
            match serde_json::from_value::<ClientConnect>(message.data) {
                Ok(event) => event.handle(&self),
                Err(_) => return,
            };
        }
    }

    fn on_disconnect(&self) {
        if !self.is_host() {
            return;
        }

        println!("Host disconnected - sending to all clients");
        self.with_podcast(|podcast| {
            // Send event to clients
            let event = CloseConnectionEvent {
                reason: close_connection_event::HOST_DISCONNECT,
            };
            podcast.send_to_all(&event);
        });
        // Remove podcast
        self.app.remove_session(self.podcast_id);
    }
}

impl Actor for PodcastWsSession {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.addr = Some(ctx.address());

        self.with_podcast(|podcast| {
            let clients = &mut podcast.ws_clients;
            clients.push(self.clone());

            let port = podcast.audio_server.port;
            let event = HelloEvent { port };

            self.send_json(&event);
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        self.on_disconnect();
    }
}

impl StreamHandler<Result<ws::Message, ProtocolError>> for PodcastWsSession {
    fn handle(&mut self, msg: Result<ws::Message, ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => return ctx.stop(),
        };

        debug!(?msg);
        match msg {
            ws::Message::Text(text_msg) => {
                println!("This = {}", text_msg.len());
                self.on_message_receive(text_msg.to_string())
            }
            ws::Message::Close(reason) => {
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
                addr: None,
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
