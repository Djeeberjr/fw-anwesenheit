use picoserve::{
    extract::{Json, State},
    response::{self, IntoResponse},
};
use serde::Deserialize;

use crate::{
    store::{Name, tally_id::TallyID},
    webserver::{app::AppState, sse::IDEvents},
};

#[derive(Deserialize)]
pub struct NewMapping {
    id: TallyID,
    name: Name,
}

// struct MappingWrapper(IDMapping);
//
// impl Serialize for MappingWrapper {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         use serde::ser::SerializeMap;
//         let mut map = serializer.serialize_map(Some(self.0.id_map.len()))?;
//         for (k, v) in &self.0.id_map {
//             map.serialize_entry(tally_id_to_hex_string(*k).unwrap().as_str(), &v)?;
//         }
//         map.end()
//     }
// }

pub async fn get_mapping(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.store.lock().await;
    response::Json(store.mapping.clone())
}

pub async fn add_mapping(
    State(state): State<AppState>,
    Json(data): Json<NewMapping>,
) -> impl IntoResponse {
    let mut store = state.store.lock().await;
    store.mapping.add_mapping(data.id, data.name);
}

pub async fn get_idevent(State(state): State<AppState>) -> impl IntoResponse {
    response::EventStream(IDEvents(state.chan.subscriber().unwrap()))
}
