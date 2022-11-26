use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct HelloEvent {
    ip: String,
}
