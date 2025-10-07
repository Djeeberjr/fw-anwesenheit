use alloc::string::String;
use picoserve::{
    extract::{Json, State},
    response::{self, IntoResponse},
};
use serde::Deserialize;

use crate::{
    store::{Name, TallyID},
    webserver::app::AppState,
};

#[derive(Deserialize)]
pub struct NewMapping {
    id: String,
    name: Name,
}

pub fn hex_string_to_tally_id(s: &str) -> Option<TallyID> {
    let bytes = s.as_bytes();
    if bytes.len() != 24 {
        return None;
    }

    let mut out = [0u8; 12];
    for i in 0..12 {
        let hi = hex_val(bytes[2 * i])?;
        let lo = hex_val(bytes[2 * i + 1])?;
        out[i] = (hi << 4) | lo;
    }
    Some(out)
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
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
