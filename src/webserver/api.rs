use esp_println::dbg;
use log::error;
use picoserve::{
    extract::{Json, Query, State},
    response::{self, IntoResponse},
};
use serde::Deserialize;

use crate::{
    store::{self, Name, day::Day, tally_id::TallyID},
    webserver::{app::AppState, sse::IDEvents},
};

#[derive(Deserialize)]
pub struct NewMapping {
    id: TallyID,
    name: Name,
}

#[derive(Deserialize)]
pub struct QueryTimespan {
    from: u64,
    to: u64,
}

#[derive(Deserialize)]
pub struct QueryDay {
    timestamp: Option<u64>,
    day: Option<u32>,
}

// GET /api/mapping
pub async fn get_mapping(State(state): State<AppState>) -> impl IntoResponse {
    let store = state.store.lock().await;
    response::Json(store.mapping.clone())
}

// POST /api/mapping
pub async fn add_mapping(
    State(state): State<AppState>,
    Json(data): Json<NewMapping>,
) -> impl IntoResponse {
    let mut store = state.store.lock().await;
    store.mapping.add_mapping(data.id, data.name);
}

// SSE /api/idevent
pub async fn get_idevent(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match state.chan.subscriber() {
        Ok(chan) => Ok(response::EventStream(IDEvents(chan))),
        Err(e) => {
            error!("Failed to create SSE: {:?}", e);
            Err((
                response::StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error",
            ))
        }
    }
}

// GET /api/days
pub async fn get_days(
    State(state): State<AppState>,
    Query(QueryTimespan { from, to }): Query<QueryTimespan>,
) -> impl IntoResponse {
    let from_day = Day::new_from_timestamp(from);
    let to_day = Day::new_from_timestamp(to);

    let mut store = state.store.lock().await;

    let days = store.list_days_in_timespan(from_day, to_day).await;

    response::Json(days)
}

// GET /api/day
pub async fn get_day(
    State(state): State<AppState>,
    Query(QueryDay { timestamp, day }): Query<QueryDay>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let parsed_day = timestamp
        .map(Day::new_from_timestamp)
        .or_else(|| day.map(Day::new))
        .ok_or((response::StatusCode::NOT_FOUND, "Not found"))?;

    let mut store = state.store.lock().await;

    match store.load_day(parsed_day).await {
        Some(att_day) => Ok(response::Json(att_day)),
        None => Err((response::StatusCode::NOT_FOUND, "Not found")),
    }
}
