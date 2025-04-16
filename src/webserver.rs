use rocket::{get, routes};

pub async fn start_webserver() -> Result<(), rocket::Error> {
    rocket::build().mount("/", routes![index]).launch().await?;
    Ok(())
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}
