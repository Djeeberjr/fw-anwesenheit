use rocket::{get, http::ContentType, response::content::RawHtml, routes};
use rust_embed::Embed;
use std::borrow::Cow;
use std::ffi::OsStr;

#[derive(Embed)]
#[folder = "web/dist"]
struct Asset;

pub async fn start_webserver() -> Result<(), rocket::Error> {
    rocket::build()
        .mount("/", routes![static_files,index])
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
