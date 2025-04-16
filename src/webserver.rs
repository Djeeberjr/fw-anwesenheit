use rocket::http::Status;
use rocket::{Config, State};
use rocket::{get, http::ContentType, response::content::RawHtml, routes};
use rust_embed::Embed;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::id_store::IDStore;

#[derive(Embed)]
#[folder = "web/dist"]
struct Asset;

pub async fn start_webserver(store: Arc<Mutex<IDStore>>) -> Result<(), rocket::Error> {
    let config = Config {
        address: "0.0.0.0".parse().unwrap(), // Listen on all interfaces
        port: 8000,
        ..Config::default()
    };

    rocket::custom(config)
        .mount("/", routes![static_files, index, export_csv])
        .manage(store)
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

#[get("/api/csv")]
async fn export_csv(manager: &State<Arc<Mutex<IDStore>>>) -> Result<String, Status> {
    match manager.lock().await.export_csv() {
        Ok(csv) => Ok(csv),
        Err(e) => {
            eprintln!("Failed to generate csv: {}", e);
            Err(Status::InternalServerError)
        }
    }
}
