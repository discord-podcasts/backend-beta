use std::collections::HashMap;
use std::sync::Mutex;

use actix_web::{App, get, HttpServer, middleware::Logger, post};
use actix_web::web::{Json, Query};
use once_cell::sync::Lazy;
use rand::prelude::IteratorRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

static PODCASTS: Lazy<Mutex<HashMap<String, Podcast>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

#[derive(Deserialize)]
struct PodcastQuery {
    id: String,
}

#[derive(Serialize, Deserialize)]
struct Podcast {
    id: String,
    active_since: Option<i32>,
}

#[get("/podcast")]
async fn get_podcast(info: Query<PodcastQuery>) -> Json<Podcast> {
    let podcast = Podcast {
        id: info.into_inner().id,
        active_since: None,
    };
    Json(podcast)
}

#[post("/podcast")]
async fn create_podcast() -> Json<Podcast> {
    let podcast = Podcast {
        id: generate_id(),
        active_since: None,
    };
    println!("{}", podcast.id);
    Json(podcast)
}

fn generate_id() -> String {
    let length = 5;
    let chars = "abcdefghijklmnopqrstuvwxyz".chars();
    let mut id = String::new();

    for _x in 0..length {
        let char = chars.clone().choose(&mut thread_rng()).unwrap();
        id.push(char);
    }
    if PODCASTS.lock().unwrap().contains_key(&id) {
        return generate_id();
    }
    return id;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .service(get_podcast)
            .service(create_podcast)
    })
        .bind(("127.0.0.1", 5050))?
        .run()
        .await
}
