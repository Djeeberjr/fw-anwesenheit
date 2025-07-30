use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_time::Duration;
use picoserve::{AppBuilder, AppRouter, routing::get};
use static_cell::make_static;

mod assets;

pub fn start_webserver(spawner: &mut Spawner, stack: Stack<'static>) {
    let app = make_static!(AppProps.build_app());

    let config = make_static!(picoserve::Config::new(picoserve::Timeouts {
        start_read_request: Some(Duration::from_secs(5)),
        persistent_start_read_request: Some(Duration::from_secs(1)),
        read_request: Some(Duration::from_secs(1)),
        write: Some(Duration::from_secs(1)),
    }));

    let _ = spawner.spawn(webserver_task(0, stack, app, config));
}

struct AppProps;

impl AppBuilder for AppProps {
    type PathRouter = impl picoserve::routing::PathRouter;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        picoserve::Router::from_service(assets::Assets).route("/api/a", get(async move || "Hello"))
    }
}

#[embassy_executor::task]
async fn webserver_task(
    id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config<Duration>,
) -> ! {
    let mut tcp_rx_buffer = [0u8; 1024];
    let mut tcp_tx_buffer = [0u8; 1024];
    let mut http_buffer = [0u8; 2048];

    picoserve::listen_and_serve(
        id,
        app,
        config,
        stack,
        80,
        &mut tcp_rx_buffer,
        &mut tcp_tx_buffer,
        &mut http_buffer,
    )
    .await
}
