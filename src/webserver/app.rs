use alloc::rc::Rc;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use picoserve::{AppWithStateBuilder, routing::get};

use crate::{
    TallyChannel, UsedStore,
    init::sd_card::SDCardPersistence,
    store::mapping_loader::MappingLoader,
    webserver::{
        api::{add_mapping, get_day, get_days, get_idevent, get_mapping, get_mappings},
        assets::Assets,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub store: Rc<Mutex<CriticalSectionRawMutex, UsedStore>>,
    pub chan: &'static TallyChannel,
    pub mapping_loader: Rc<Mutex<CriticalSectionRawMutex, MappingLoader<SDCardPersistence>>>,
}

pub struct AppProps;

impl AppWithStateBuilder for AppProps {
    type State = AppState;
    type PathRouter = impl picoserve::routing::PathRouter<AppState>;

    fn build_app(self) -> picoserve::Router<Self::PathRouter, AppState> {
        picoserve::Router::from_service(Assets)
            .route("/api/mapping", get(get_mapping).post(add_mapping))
            .route("/api/mappings", get(get_mappings))
            .route("/api/idevent", get(get_idevent))
            .route("/api/days", get(get_days))
            .route("/api/day", get(get_day))
    }
}
