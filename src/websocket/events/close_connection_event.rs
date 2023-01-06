use actix::{Handler, Message};
use serde::Serialize;

use crate::websocket::ws::PodcastWsSession;

use super::events::Event;

/// When the host disconnects and the podcast gets stopped
#[derive(Serialize, Clone, Message)]
#[rtype(result = "()")]
pub struct CloseConnectionEvent {
    reason: u16,
}

pub static HOST_DISCONNECT: u16 = 0;

impl Handler<CloseConnectionEvent> for PodcastWsSession {
    type Result = ();

    fn handle(&mut self, msg: CloseConnectionEvent, ctx: &mut Self::Context) {
        msg.send_to_client(ctx);
    }
}

impl Event for CloseConnectionEvent {
    fn event_type(&self) -> u32 {
        2
    }
}
