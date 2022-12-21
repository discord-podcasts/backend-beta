use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EventWrapper<T: Serialize> {
    pub event_type: u32,
    pub data: T,
}
pub trait Event: Serialize {
    fn event_type(&self) -> u32;

    fn data(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

// Event implementations

#[derive(Serialize)]
pub struct HelloEvent {
    pub port: u16,
}

impl Event for HelloEvent {
    fn event_type(&self) -> u32 {
        1
    }
}
