use buzzer::GPIOBuzzer;
use id_store::IDStore;
use led::Led;
use log::{LevelFilter, debug, error, info, warn};
use pm3::run_pm3;
use rppal::pwm::Channel;
use simplelog::{ConfigBuilder, SimpleLogger};
use std::{env, error::Error, sync::Arc};
use tokio::{
    fs,
    sync::{Mutex, mpsc},
};
use webserver::start_webserver;

mod buzzer;
mod color;
mod id_store;
mod led;
mod parser;
mod pm3;
mod webserver;

const STORE_PATH: &str = "./data.json";
const PWM_CHANNEL_BUZZER: Channel = Channel::Pwm0; //PWM0 = GPIO18/Physical pin 12

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
        .set_target_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .build();

    let _ = SimpleLogger::init(log_level, config);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    let raw_store = if fs::try_exists(STORE_PATH).await? {
        info!("Loading data from file");
        IDStore::new_from_json(STORE_PATH).await?
    } else {
        info!("No data file found. Creating empty one.");
        IDStore::new()
    };

    debug!("created store sucessfully");

    let store: Arc<Mutex<IDStore>> = Arc::new(Mutex::new(raw_store));
    let gpio_buzzer: Arc<Mutex<GPIOBuzzer>> =
        Arc::new(Mutex::new(GPIOBuzzer::new(PWM_CHANNEL_BUZZER)?));
    let status_led: Arc<Mutex<Led>> = Arc::new(Mutex::new(Led::new()?));

    let channel_store = store.clone();
    tokio::spawn(async move {
        while let Some(tally_id_string) = rx.recv().await {
            if channel_store
                .lock()
                .await
                .add_id(id_store::TallyID(tally_id_string))
            {
                info!("Added new id to current day");
                status_led
                    .lock()
                    .await
                    .turn_green_on_1s()
                    .await
                    .unwrap_or_else(|e| {
                        error!("Failed to blink LED {}", e);
                    });

                gpio_buzzer
                    .lock()
                    .await
                    .beep_ack()
                    .await
                    .unwrap_or_else(|e| error!("Failed to beep Ack {}", e));

                if let Err(e) = channel_store.lock().await.export_json(STORE_PATH).await {
                    error!("Failed to save id store to file: {}", e);
                    // TODO: How to handle a failure to save ?
                    gpio_buzzer
                        .lock()
                        .await
                        .beep_nak()
                        .await
                        .unwrap_or_else(|e| error!("Failed to beep Nack {}", e));
                }
            }
        }
    });

    match start_webserver(store.clone()).await {
        Ok(()) => {}
        Err(e) => {
            error!("Failed to start webserver: {}", e);
        }
    }

    Ok(())
}
