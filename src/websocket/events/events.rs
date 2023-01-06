use crate::websocket::ws::PodcastWsSession;
use actix_web_actors::ws::WebsocketContext;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EventWrapper<T: Serialize> {
    pub event_type: u32,
    pub data: T,
}

pub trait Event
where
    Self: Clone + Serialize + actix::Message,
{
    fn event_type(&self) -> u32;

    fn send_to_client(&self, ctx: &WebsocketContext<PodcastWsSession>) {
        let wrapper = EventWrapper {
            event_type: self.event_type(),
            data: self,
        };
        let json = serde_json::to_string(&wrapper).unwrap();
        ctx.text(json);
    }
}
