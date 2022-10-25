use std::collections::HashMap;

use actix_web::{App, get, HttpResponse, HttpServer, middleware::Logger, post, Responder, web::Data};
use actix_web::web::{Json, Query};
use serde::{Deserialize, Serialize};
use serde::__private::de::Content::String;

#[derive(Deserialize)]
struct PodcastQuery {
    id: String,
}

#[derive(Serialize, Deserialize)]
struct Podcast {
    id: String,
    active_since: Option<i32>,
}

#[get("/")]
async fn create_podcast(info: Query<PodcastQuery>) -> Json<Podcast> {
    let podcast = Podcast {
        id: info.into_inner().id,
        active_since: None,
    };
    Json(podcast)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let podcasts: HashMap<String, Podcast> = HashMap::new();

    HttpServer::new(move || {
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .service(create_podcast)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}