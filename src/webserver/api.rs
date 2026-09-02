use log::error;
use picoserve::{
    extract::{Json, Query, State},
    response::{self, IntoResponse},
};
use serde::Deserialize;

use crate::{
    store::{day::Day, mapping_loader::Name, tally_id::TallyID},
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

#[derive(Deserialize)]
pub struct QueryMapping {
    id: TallyID,
}

// GET /api/mappings
pub async fn get_mappings(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let loader = state.mapping_loader.lock().await;

    match loader.list_mappings().await {
        Ok(ids) => Ok(response::Json(ids)),
        Err(_) => Err((
            response::StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_SERVER_ERROR",
        )),
    }
}

// GET /api/mapping
pub async fn get_mapping(
    State(state): State<AppState>,
    Query(QueryMapping { id }): Query<QueryMapping>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let loader = state.mapping_loader.lock().await;

    match loader.get_mapping(id).await {
        Ok(name) => Ok(response::Json(name)),
        Err(_) => Err((
            response::StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_SERVER_ERROR",
        )),
    }
}

// POST /api/mapping
pub async fn add_mapping(
    State(state): State<AppState>,
    Json(data): Json<NewMapping>,
) -> impl IntoResponse {
    let loader = state.mapping_loader.lock().await;
    match loader.set_mapping(data.id, data.name).await {
        Ok(_) => (response::StatusCode::CREATED, ""),
        Err(_) => (
            response::StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_SERVER_ERROR",
        ),
    }
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

    match store.list_days_in_timespan(from_day, to_day).await {
        Ok(days) => Ok(response::Json(days)),
        Err(_) => Err((
            response::StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
        )),
    }
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
        Ok(att_day) => Ok(response::Json(att_day)),
        Err(_) => Err((
            response::StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
        )),
    }
}

// GET /api/time
pub async fn get_time(State(state): State<AppState>) -> impl IntoResponse {
    let time = state.rtc.lock().await.get_time().await;
    response::Json(time)
}

// POST /api/time
pub async fn set_time(State(state): State<AppState>, Json(data): Json<u64>) -> impl IntoResponse {
    state.rtc.lock().await.set_time(data).await;
    response::StatusCode::NO_CONTENT
}
