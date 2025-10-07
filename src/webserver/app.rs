use alloc::rc::Rc;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use picoserve::{AppWithStateBuilder, routing::get};

use crate::{
    TallyChannel, UsedStore,
    webserver::{
        api::{add_mapping, get_idevent, get_mapping},
        assets::Assets,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub store: Rc<Mutex<CriticalSectionRawMutex, UsedStore>>,
    pub chan: &'static TallyChannel,
}

pub struct AppProps;

impl AppWithStateBuilder for AppProps {
    type State = AppState;
    type PathRouter = impl picoserve::routing::PathRouter<AppState>;

    fn build_app(self) -> picoserve::Router<Self::PathRouter, AppState> {
        picoserve::Router::from_service(Assets)
            .route("/api/mapping", get(get_mapping).post(add_mapping))
            .route("/api/idevent", get(get_idevent))
    }
}
