use std::any::type_name;

use actix::{Actor, ActorContext, StreamHandler};
use actix_web::{
    body::BoxBody,
    http::StatusCode,
    web::{Data, Payload, Query},
    HttpRequest, HttpResponse,
};
use actix_web_actors::ws::{self, Message, ProtocolError, WebsocketContext};
use tracing::debug;

use crate::{
    events::{Event, EventWrapper, HelloEvent},
    podcast::{PodcastData, PodcastQuery},
    Application,
};

struct PodcastWs {
    id: u32,
    app: Data<Application>,
}

impl PodcastWs {
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
            .with_session(self.id, |session| session.data.clone())
            .unwrap()
    }

    fn get_audio_server_port(&self) -> u16 {
        self.app
            .with_session(self.id, |session| session.audio_server_port)
            .unwrap()
    }
}

impl Actor for PodcastWs {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let event = HelloEvent {
            port: self.get_audio_server_port(),
        };
        self.send_json(ctx, &event);
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for PodcastWs {
    fn handle(&mut self, msg: Result<ws::Message, ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => return ctx.stop(),
        };

        debug!(?msg);
        match msg {
            Message::Text(_) => {}
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
    let addr = match req.peer_addr() {
        Some(addr) => addr,
        None => {
            debug!("Rejecting websocket connection without peer address");
            return Ok(HttpResponse::new(StatusCode::BAD_REQUEST)
                .set_body(BoxBody::new("Missing peer address")));
        }
    };

    debug!(?addr, "Incoming websocket connection");
    let podcast = app.with_session(query.id, |session| session.data.clone());
    match podcast {
        Some(podcast) => {
            let podcast_ws = PodcastWs {
                id: podcast.id,
                app,
            };
            ws::start(podcast_ws, &req, stream)
        }
        None => Err(actix_web::error::ErrorNotFound("Podcast doesn't exist")),
    }
}
