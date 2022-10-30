use actix_web::web::{Data, Json, Query};
use serde::{Deserialize, Serialize};

use crate::Application;

#[derive(Serialize, Deserialize)]
pub struct Podcast {
    pub id: String,
    pub active_since: Option<i32>,
}

#[derive(Deserialize)]
pub struct PodcastQuery {
    id: String,
}

pub async fn get(Query(query): Query<PodcastQuery>) -> Json<Podcast> {
    let podcast = Podcast {
        id: query.id,
        active_since: None,
    };
    Json(podcast)
}

pub async fn create(app: Data<Application>) -> Json<Podcast> {
    let podcast = Podcast {
        id: app.generate_id(),
        active_since: None,
    };
    Json(podcast)
}
