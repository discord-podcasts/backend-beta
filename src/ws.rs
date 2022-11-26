use actix::{Actor, ActorContext, StreamHandler};
use actix_web::{
    body::BoxBody,
    http::StatusCode,
    web::{Data, Payload, Query},
    HttpRequest, HttpResponse,
};
use actix_web_actors::ws::{self, Message, ProtocolError, WebsocketContext};
use serde::Serialize;
use tracing::debug;

use crate::{events::HelloEvent, podcast::PodcastQuery, Application};

struct PodcastWs {
    id: u32,
}

impl PodcastWs {
    fn send_json<T>(&self, ctx: &mut WebsocketContext<Self>, value: &T)
    where
        T: ?Sized + Serialize,
    {
        let text = serde_json::to_string(value);
        match text {
            Ok(text) => ctx.text(text),
            Err(_) => {}
        }
    }
}

impl Actor for PodcastWs {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Access Application here?
        let ip_test = String::from("test");
        let event = &HelloEvent { ip: ip_test };
        self.send_json(ctx, event);
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
    let podcast = app.get_session(&query.id);
    match podcast {
        Some(podcast) => {
            let podcastWs = PodcastWs {
                id: podcast.data.id,
            };
            ws::start(podcastWs, &req, stream)
        }
        None => Err(actix_web::error::ErrorNotFound("Podcast doesn't exist")),
    }
}
