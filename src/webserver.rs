use log::{error, info, warn};
use rocket::http::Status;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::json::Json;
use rocket::{Config, State, post};
use rocket::{get, http::ContentType, response::content::RawHtml, routes};
use rust_embed::Embed;
use serde::Deserialize;
use std::borrow::Cow;
use std::env;
use std::ffi::OsStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::broadcast::Sender;

use crate::id_mapping::{IDMapping, Name};
use crate::id_store::IDStore;
use crate::tally_id::TallyID;

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
fn id_event(sse_broadcaster: &State<Sender<String>>) -> EventStream![] {
    let mut rx = sse_broadcaster.subscribe();
    EventStream! {
        loop {
            let msg = rx.recv().await;
            if let Ok(id) = msg {
                yield Event::data(id);
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
async fn add_mapping(store: &State<Arc<Mutex<IDStore>>>, new_mapping: Json<NewMapping>) {
    store
        .lock()
        .await
        .mapping
        .add_mapping(TallyID(new_mapping.id.clone()), new_mapping.name.clone());
}
