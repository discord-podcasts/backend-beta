use actix::{Actor, ActorContext, StreamHandler};
use actix_web::{
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
    // FIXME: Handle request without peer addr
    let addr = req.peer_addr().unwrap();
    debug!(?addr, "Incoming websocket connection");
    let podcast = PodcastWs {};
    ws::start(podcast, &req, stream)
}
