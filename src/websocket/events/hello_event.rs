use super::events::Event;
use crate::websocket::ws::PodcastWsSession;
use actix::{Handler, Message};
use serde::Serialize;

/// Send when a new client connects
#[derive(Serialize, Message, Clone)]
#[rtype(result = "()")]
pub struct HelloEvent {
    pub port: u16, // Audio socket port
}

impl Handler<HelloEvent> for PodcastWsSession {
    type Result = ();

    fn handle(&mut self, msg: HelloEvent, ctx: &mut Self::Context) {
        msg.send_to_client(ctx);
    }
}

impl Event for HelloEvent {
    fn event_type(&self) -> u32 {
        1
    }
}
