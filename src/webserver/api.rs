use alloc::string::String;
use picoserve::{
    extract::{Json, State},
    response::{self, IntoResponse},
};
use serde::Deserialize;
use crate::{
    store::{Name, hex_string_to_tally_id},
    webserver::{app::AppState, sse::IDEvents},
};

#[derive(Deserialize)]
pub struct NewMapping {
    id: String,
    name: Name,
}

/*
 *  #[get("/api/idevent")]
 *  #[get("/api/csv")]
 *  #[get("/api/mapping")]
 *  #[post("/api/mapping", format = "json", data = "<new_mapping>")]
 *  struct NewMapping {
 *      id: String,
 *      name: Name,
 *  }
*/

pub async fn get_mapping(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.store.lock().await;
    response::Json(store.mapping.clone())
}

pub async fn add_mapping(
    State(state): State<AppState>,
    Json(data): Json<NewMapping>,
) -> impl IntoResponse {
    let mut store = state.store.lock().await;
    let tally_id = hex_string_to_tally_id(&data.id).unwrap();
    store.mapping.add_mapping(tally_id, data.name);
}

pub async fn get_idevent(State(state): State<AppState>) -> impl IntoResponse{
    response::EventStream(IDEvents(state.chan.subscriber().unwrap()))
}
