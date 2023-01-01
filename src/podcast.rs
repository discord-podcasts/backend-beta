use std::{
    thread,
    time::{Duration, SystemTime},
};

use actix_web::{
    error,
    web::{Data, Json, Query},
    HttpRequest,
};

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{audio_server::AudioServer, authentication::validate_authentication_data, Application};

pub struct Podcast {
    pub data: PodcastData,
    pub audio_server: AudioServer,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PodcastData {
    pub id: u32,
    pub active_since: Option<u128>,
    pub host: u32,
}

#[derive(Deserialize)]
pub struct PodcastQuery {
    pub id: u32,
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

pub async fn list(app: Data<Application>) -> Result<Json<Vec<PodcastData>>, actix_web::Error> {
    Result::Ok(Json(app.list_sessions()))
}

pub async fn create(
    app: Data<Application>,
    req: HttpRequest,
) -> Result<Json<PodcastData>, actix_web::Error> {
    let auth = match validate_authentication_data(&req, &app) {
        Ok(auth) => auth,
        Err(error) => return Err(error),
    };

    let addr = match req.peer_addr() {
        Some(addr) => addr,
        None => {
            debug!("Rejecting websocket connection without peer address");
            return Err(error::ErrorBadRequest("Missing peer address"));
        }
    };
    println!("Request by {}", addr.ip().to_string());

    let audio_server = match AudioServer::create(&app) {
        Some(audio_server) => audio_server,
        None => return Err(error::ErrorBadRequest("All possible sockets are in use")),
    };

    let podcast = Podcast {
        data: PodcastData {
            id: app.generate_id(),
            active_since: None,
            host: auth.client_id,
        },
        audio_server,
    };
    let copied_podcast_data = podcast.data.clone();

    await_host(podcast.data.id, app.clone());
    app.add_session(podcast);
    Ok(Json(copied_podcast_data))
}

/**
 * Makes sure that the host connects to its podcast in time.
 */
fn await_host(podcast_id: u32, app: Data<Application>) {
    /**
     * Checks wheter the host is connected.
     *
     * Option Some(bool): Podcast exists and bool shows wheter host is connected or not,
     * Option None: Podcast doesn't exist anymore,
     */
    fn is_host_connected(podcast_id: u32, app: Data<Application>) -> Option<bool> {
        return app.with_session(podcast_id, |session| session.data.active_since.is_some());
    }

    thread::spawn(move || {
        let start = SystemTime::now();
        loop {
            let host_connected = match is_host_connected(podcast_id, app.clone()) {
                Some(connected) => connected,
                None => return, // Podcast doesn't exist anymore
            };

            if host_connected {
                return;
            }

            // Host didn't show up after 60 seconds
            if start.elapsed().unwrap().as_secs() > 60 {
                app.remove_session(podcast_id);
                return;
            }

            thread::sleep(Duration::from_secs(2));
        }
    });
}
