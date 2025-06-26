use log::{error, info, warn};
use rocket::http::Status;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::json::Json;
use rocket::{Config, Shutdown, State, post};
use rocket::{get, http::ContentType, response::content::RawHtml, routes};
use rust_embed::Embed;
use serde::Deserialize;
use std::borrow::Cow;
use std::env;
use std::ffi::OsStr;
use std::sync::Arc;
use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::broadcast::Sender;

use crate::store::{IDMapping, IDStore, Name};
use crate::tally_id::TallyID;
use crate::webserver::ActivityNotifier;

#[derive(Embed)]
#[folder = "web/dist"]
struct Asset;

#[derive(Deserialize)]
struct NewMapping {
    id: String,
    name: Name,
}

pub async fn start_webserver(
    store: Arc<Mutex<IDStore>>,
    sse_broadcaster: Sender<String>,
    fairing: ActivityNotifier,
) -> Result<(), rocket::Error> {
    let port = match env::var("HTTP_PORT") {
        Ok(port) => port.parse().unwrap_or_else(|_| {
            warn!("Failed to parse HTTP_PORT. Using default 80");
            80
        }),
        Err(_) => 80,
    };

    let config = Config {
        address: "0.0.0.0".parse().unwrap(), // Listen on all interfaces
        port,
        ..Config::default()
    };

    rocket::custom(config)
        .attach(fairing)
        .mount(
            "/",
            routes![
                static_files,
                index,
                export_csv,
                id_event,
                get_mapping,
                add_mapping
            ],
        )
        .manage(store)
        .manage(sse_broadcaster)
        .launch()
        .await?;
    Ok(())
}

#[get("/")]
fn index() -> Option<RawHtml<Cow<'static, [u8]>>> {
    let asset = Asset::get("index.html")?;
    Some(RawHtml(asset.data))
}

#[get("/<file..>")]
fn static_files(file: std::path::PathBuf) -> Option<(ContentType, Vec<u8>)> {
    let filename = file.display().to_string();
    let asset = Asset::get(&filename)?;
    let content_type = file
        .extension()
        .and_then(OsStr::to_str)
        .and_then(ContentType::from_extension)
        .unwrap_or(ContentType::Bytes);

    Some((content_type, asset.data.into_owned()))
}

#[get("/api/idevent")]
fn id_event(sse_broadcaster: &State<Sender<String>>, shutdown: Shutdown) -> EventStream![] {
    let mut rx = sse_broadcaster.subscribe();
    EventStream! {
        loop {
            select! {
                msg = rx.recv() => {
                    if let Ok(id) = msg {
                        yield Event::data(id);
                    }
                }
                _ = &mut shutdown.clone() => {
                    // Shutdown signal received, exit the loop
                    break;
                }
            }
        }
    }
}

#[get("/api/csv")]
async fn export_csv(manager: &State<Arc<Mutex<IDStore>>>) -> Result<String, Status> {
    info!("Exporting CSV");
    match manager.lock().await.export_csv() {
        Ok(csv) => Ok(csv),
        Err(e) => {
            error!("Failed to generate csv: {e}");
            Err(Status::InternalServerError)
        }
    }
}

#[get("/api/mapping")]
async fn get_mapping(store: &State<Arc<Mutex<IDStore>>>) -> Json<IDMapping> {
    Json(store.lock().await.mapping.clone())
}

#[post("/api/mapping", format = "json", data = "<new_mapping>")]
async fn add_mapping(store: &State<Arc<Mutex<IDStore>>>, new_mapping: Json<NewMapping>) -> Status {
    if new_mapping.id.is_empty()
        || new_mapping.name.first.is_empty()
        || new_mapping.name.last.is_empty()
    {
        return Status::BadRequest;
    }

    store
        .lock()
        .await
        .mapping
        .add_mapping(TallyID(new_mapping.id.clone()), new_mapping.name.clone());

    Status::Created
}
