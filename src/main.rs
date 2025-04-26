use id_store::IDStore;
use log::{LevelFilter, error, info, warn};
use pm3::{pm3_mock, run_pm3};
use simplelog::{ConfigBuilder, SimpleLogger};
use std::{env, sync::Arc};
use tokio::sync::{Mutex, mpsc};
use webserver::start_webserver;

mod id_store;
mod parser;
mod pm3;
mod webserver;

fn setup_logger() {
    let log_level = env::var("LOG_LEVEL")
        .ok()
        .and_then(|level| level.parse::<LevelFilter>().ok())
        .unwrap_or({
            if cfg!(debug_assertions) {
                LevelFilter::Debug
            } else {
                LevelFilter::Warn
            }
        });

    let config = ConfigBuilder::new()
        .set_target_level(LevelFilter::Error)
        .build();

    let _ = SimpleLogger::init(log_level, config);
}

#[tokio::main]
async fn main() {
    setup_logger();

    info!("Starting application");

    let (tx, mut rx) = mpsc::channel::<String>(1);

    tokio::spawn(async move {
        match run_pm3(tx).await {
            Ok(()) => {
                warn!("PM3 exited with a zero exit code");
            }
            Err(e) => {
                error!("Failed to run PM3: {}", e);
            }
        }
    });

    let store:Arc<Mutex<IDStore>> = Arc::new(Mutex::new(id_store::IDStore::new()));
    let channel_store = store.clone();

    tokio::spawn(async move {
        while let Some(tally_id_string) = rx.recv().await {
            channel_store.lock().await.add_id(id_store::TallyID(tally_id_string));
        }
    });

    match start_webserver(store.clone()).await {
        Ok(()) => {}
        Err(e) => {
            error!("Failed to start webserver: {}", e);
        }
    }
}
