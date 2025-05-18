use buzzer::{Buzzer, GPIOBuzzer};
use hotspot::{Hotspot, HotspotError, NMHotspot};
use id_store::IDStore;
use led::{SpiLed, StatusLed};
use log::{LevelFilter, debug, error, info, warn};
use pm3::run_pm3;
use rppal::pwm::Channel;
use simplelog::{ConfigBuilder, SimpleLogger};
use std::{env, error::Error, sync::Arc};
use tally_id::TallyID;
use tokio::{
    fs, join,
    sync::{Mutex, broadcast, mpsc},
};
use webserver::start_webserver;

#[cfg(feature = "mock_pi")]
use mock::{MockBuzzer, MockHotspot, MockLed};

mod buzzer;
mod color;
mod hotspot;
mod id_mapping;
mod id_store;
mod led;
mod mock;
mod parser;
mod pm3;
mod tally_id;
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

/// Signal the user success via buzzer and led
async fn feedback_success<T: Buzzer, I: StatusLed>(
    gpio_buzzer: &Arc<Mutex<T>>,
    status_led: &Arc<Mutex<I>>,
) {
    let mut buzzer_guard = gpio_buzzer.lock().await;
    let mut led_guard = status_led.lock().await;

    let (buzz, led) = join!(buzzer_guard.beep_ack(), led_guard.turn_green_on_1s());

    buzz.unwrap_or_else(|err| {
        error!("Failed to buzz: {err}");
    });

    led.unwrap_or_else(|err| {
        error!("Failed to set LED: {err}");
    });
}

/// Signal the user failure via buzzer and led
async fn feedback_failure<T: Buzzer, I: StatusLed>(
    gpio_buzzer: &Arc<Mutex<T>>,
    status_led: &Arc<Mutex<I>>,
) {
    let mut buzzer_guard = gpio_buzzer.lock().await;
    let mut led_guard = status_led.lock().await;

    let (buzz, led) = join!(buzzer_guard.beep_nak(), led_guard.turn_red_on_1s());

    buzz.unwrap_or_else(|err| {
        error!("Failed to buzz: {err}");
    });

    led.unwrap_or_else(|err| {
        error!("Failed to set LED: {err}");
    });
}

/// Create a buzzer
/// Respects the `mock_pi` flag.
fn create_buzzer() -> Result<Arc<Mutex<impl Buzzer>>, rppal::pwm::Error> {
    #[cfg(feature = "mock_pi")]
    {
        Ok(Arc::new(Mutex::new(MockBuzzer {})))
    }

    #[cfg(not(feature = "mock_pi"))]
    {
        Ok(Arc::new(Mutex::new(GPIOBuzzer::new(PWM_CHANNEL_BUZZER)?)))
    }
}

/// Creates a status led.
/// Respects the `mock_pi` flag.
fn create_status_led() -> Result<Arc<Mutex<impl StatusLed>>, rppal::spi::Error> {
    #[cfg(feature = "mock_pi")]
    {
        Ok(Arc::new(Mutex::new(MockLed {})))
    }

    #[cfg(not(feature = "mock_pi"))]
    {
        Ok(Arc::new(Mutex::new(SpiLed::new()?)))
    }
}

/// Create a struct to manage the hotspot
/// Respects the `mock_pi` flag.
fn create_hotspot() -> Result<impl Hotspot, HotspotError> {
    #[cfg(feature = "mock_pi")]
    {
        Ok(MockHotspot {})
    }

    #[cfg(not(feature = "mock_pi"))]
    {
        NMHotspot::new_from_env()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    info!("Starting application");

    let (tx, mut rx) = broadcast::channel::<String>(32);
    let sse_tx = tx.clone();

    tokio::spawn(async move {
        match run_pm3(tx).await {
            Ok(()) => {
                warn!("PM3 exited with a zero exit code");
            }
            Err(e) => {
                error!("Failed to run PM3: {e}");
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
    let gpio_buzzer = create_buzzer()?;
    let status_led = create_status_led()?;
    let hotspot = create_hotspot()?;

    let hotspot_ids: Vec<TallyID> = env::var("HOTSPOT_IDS")
        .map(|ids| ids.split(";").map(|id| TallyID(id.to_owned())).collect())
        .unwrap_or_default();

    if hotspot_ids.is_empty() {
        warn!(
            "HOTSPOT_IDS is not set or empty. You will not be able to activate the hotspot via a tally!"
        );
    }

    let channel_store = store.clone();
    tokio::spawn(async move {
        while let Ok(tally_id_string) = rx.recv().await {
            let tally_id = TallyID(tally_id_string);

            if hotspot_ids.contains(&tally_id) {
                info!("Enableing hotspot");
                hotspot.enable_hotspot().await.unwrap_or_else(|err| {
                    error!("Hotspot: {err}");
                });
                // TODO: Should the ID be added anyway or ignored ?
            }

            if channel_store.lock().await.add_id(tally_id) {
                info!("Added new id to current day");

                feedback_success(&gpio_buzzer, &status_led).await;

                if let Err(e) = channel_store.lock().await.export_json(STORE_PATH).await {
                    error!("Failed to save id store to file: {e}");
                    feedback_failure(&gpio_buzzer, &status_led).await;
                    // TODO: How to handle a failure to save ?
                }
            }
        }
    });

    match start_webserver(store.clone(), sse_tx).await {
        Ok(()) => {}
        Err(e) => {
            error!("Failed to start webserver: {e}");
        }
    }

    Ok(())
}
