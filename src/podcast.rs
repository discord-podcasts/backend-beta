use std::{net::UdpSocket, sync::Arc, thread, time::SystemTime};

use actix_web::{
    error,
    web::{Data, Json, Query},
};
use rand::distributions::uniform::SampleBorrow;
use serde::{Deserialize, Serialize};

use crate::{audio_server::AudioServer, Application};

pub struct Podcast {
    pub data: PodcastData,
    pub audio_server: UdpSocket,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PodcastData {
    pub id: u32,
    pub active_since: Option<i32>,
}

#[derive(Deserialize)]
pub struct PodcastQuery {
    id: u32,
}

pub async fn get(
    Query(query): Query<PodcastQuery>,
    app: Data<Application>,
) -> Result<Json<PodcastData>, actix_web::Error> {
    let id = query.id;

    match app.sessions.lock().unwrap().get(&id) {
        Some(podcast) => Ok(Json(podcast.data.clone())),
        None => Err(error::ErrorNotFound("Podcast doesn't exist")),
    }
}

pub async fn create(app: Data<Application>) -> Result<Json<PodcastData>, actix_web::Error> {
    let audio_server = match AudioServer::create(&app) {
        Some(audio_server) => audio_server,
        None => return Err(error::ErrorBadRequest("All possible sockets are in use")),
    };
    println!(
        "Created audio server at 127.0.0.1:{}",
        audio_server.local_addr().unwrap().port()
    );

    let podcast = Podcast {
        data: PodcastData {
            id: app.generate_id(),
            active_since: None,
        },
        audio_server: audio_server,
    };

    let podcast_data = podcast.data.clone();
    app.add_session(podcast);

    let thread_safe_podcast = Arc::from(podcast_data.clone());
    await_host(thread_safe_podcast, app);

    Ok(Json(podcast_data))
}

pub async fn list(app: Data<Application>) -> Result<Json<Vec<PodcastData>>, actix_web::Error> {
    Result::Ok(Json(app.list_sessions()))
}

fn await_host(podcast: Arc<PodcastData>, app: Data<Application>) {
    thread::spawn(move || {
        let start = SystemTime::now();
        println!("hi");
        while podcast.active_since.is_none() {
            if start.elapsed().unwrap().as_secs() > 60 {
                app.remove_session(podcast.id.borrow());
                return;
            }
        }
    });
}
