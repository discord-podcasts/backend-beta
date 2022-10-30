use std::collections::HashMap;
use std::env;

use actix::{Actor, Context};
use actix_web::web::{self, Data};
use actix_web::{middleware::Logger, App, HttpServer};
use rand::Rng;
use tracing_subscriber::EnvFilter;

use crate::podcast::Podcast;

mod podcast;
mod ws;

pub struct Application {
    sessions: HashMap<u32, Podcast>,
}

impl Application {
    fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    fn generate_id(&self) -> u32 {
        let id: u32 = rand::thread_rng().gen();
        if self.sessions.contains_key(&id) {
            return self.generate_id();
        }
        id
    }
}

impl Actor for Application {
    type Context = Context<Application>;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // FIXME: Set fallback port
    let port: u16 = env::var("PORT").unwrap().parse().unwrap();

    let app = Data::new(Application::new());
    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .route("/podcast", web::get().to(podcast::get))
            .route("/podcast", web::post().to(podcast::create))
            .route("/ws", web::get().to(ws::websocket))
            .app_data(Data::clone(&app))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
