use alloc::rc::Rc;
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Duration;
use picoserve::{AppRouter, AppWithStateBuilder};
use static_cell::make_static;

use crate::{
    UsedStore,
    webserver::app::{AppProps, AppState},
};

mod assets;
// mod sse;
mod api;
mod app;

pub const WEB_TAKS_SIZE: usize = 3; // Up this number if request start fail with Timeouts.

pub fn start_webserver(
    spawner: &mut Spawner,
    stack: Stack<'static>,
    store: Rc<Mutex<CriticalSectionRawMutex, UsedStore>>,
) {
    let app = make_static!(AppProps.build_app());

    let state = make_static!(AppState { store });

    let config = make_static!(picoserve::Config::new(picoserve::Timeouts {
        start_read_request: Some(Duration::from_secs(5)),
        persistent_start_read_request: Some(Duration::from_secs(5)),
        read_request: Some(Duration::from_secs(5)),
        write: Some(Duration::from_secs(5)),
    }));

    for task_id in 0..WEB_TAKS_SIZE {
        spawner.must_spawn(webserver_task(task_id, stack, app, config, state));
    }
}

#[embassy_executor::task(pool_size = WEB_TAKS_SIZE)]
async fn webserver_task(
    task_id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config<Duration>,
    state: &'static AppState,
) -> ! {
    let mut tcp_rx_buffer = [0u8; 1024];
    let mut tcp_tx_buffer = [0u8; 1024];
    let mut http_buffer = [0u8; 2048];

    picoserve::Server::new(&app.shared().with_state(state), config, &mut http_buffer)
        .listen_and_serve(task_id, stack, 80, &mut tcp_rx_buffer, &mut tcp_tx_buffer)
        .await
        .into_never()
}
