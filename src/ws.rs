use actix::{Actor, ActorContext, StreamHandler};
use actix_web::{
    body::BoxBody,
    http::StatusCode,
    web::{Data, Payload},
    HttpRequest, HttpResponse,
};
use actix_web_actors::ws::{self, Message, ProtocolError, WebsocketContext};
use tracing::debug;

use crate::Application;

struct PodcastWs {}

impl Actor for PodcastWs {
    type Context = WebsocketContext<Self>;
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
    req: HttpRequest,
    stream: Payload,
    _app: Data<Application>,
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
    let podcast = PodcastWs {};
    ws::start(podcast, &req, stream)
}
