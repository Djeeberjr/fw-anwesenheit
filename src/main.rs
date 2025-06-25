#![allow(dead_code)]

use activity_fairing::{ActivityNotifier, spawn_idle_watcher};
use anyhow::Result;
use feedback::{Feedback, FeedbackImpl};
use hardware::{Hotspot, create_hotspot};
use id_store::IDStore;
use log::{error, info, warn};
use pm3::run_pm3;
use std::{
    env::{self, args},
    sync::Arc,
    time::Duration,
};
use tally_id::TallyID;
use tokio::{
    fs,
    signal::unix::{SignalKind, signal},
    sync::{
        Mutex,
        broadcast::{self, Receiver, Sender},
    },
    try_join,
};
use webserver::start_webserver;

mod activity_fairing;
mod feedback;
mod gpio_buzzer;
mod hardware;
mod hotspot;
mod id_mapping;
mod id_store;
mod logger;
mod mock;
mod parser;
mod pm3;
mod spi_led;
mod tally_id;
mod webserver;

const STORE_PATH: &str = "./data.json";

async fn run_webserver<H>(
    store: Arc<Mutex<IDStore>>,
    id_channel: Sender<String>,
    hotspot: Arc<Mutex<H>>,
    user_feedback: Arc<Mutex<FeedbackImpl>>,
) -> Result<()>
where
    H: Hotspot + Send + Sync + 'static,
{
    let activity_channel = spawn_idle_watcher(Duration::from_secs(60 * 30), move || {
        info!("No activity on webserver. Disabling hotspot");
        let cloned_hotspot = hotspot.clone();
        let cloned_user_feedback = user_feedback.clone();
        tokio::spawn(async move {
            let _ = cloned_hotspot.lock().await.disable_hotspot().await;
            cloned_user_feedback
                .lock()
                .await
                .set_device_status(feedback::DeviceStatus::Ready);
        });
    });

    let notifier = ActivityNotifier {
        sender: activity_channel,
    };

    start_webserver(store, id_channel, notifier).await?;

    Ok(())
}

async fn load_or_create_store() -> Result<IDStore> {
    if fs::try_exists(STORE_PATH).await? {
        info!("Loading data from file");
        IDStore::new_from_json(STORE_PATH).await
    } else {
        info!("No data file found. Creating empty one.");
        Ok(IDStore::new())
    }
}

fn get_hotspot_enable_ids() -> Vec<TallyID> {
    let hotspot_ids: Vec<TallyID> = env::var("HOTSPOT_IDS")
        .map(|ids| ids.split(";").map(|id| TallyID(id.to_owned())).collect())
        .unwrap_or_default();

    if hotspot_ids.is_empty() {
        warn!(
            "HOTSPOT_IDS is not set or empty. You will not be able to activate the hotspot via a tally!"
        );
    }

    hotspot_ids
}

async fn handle_ids_loop(
    mut id_channel: Receiver<String>,
    hotspot_enable_ids: Vec<TallyID>,
    id_store: Arc<Mutex<IDStore>>,
    hotspot: Arc<Mutex<impl Hotspot>>,
    user_feedback: Arc<Mutex<FeedbackImpl>>,
) -> Result<()> {
    while let Ok(tally_id_string) = id_channel.recv().await {
        let tally_id = TallyID(tally_id_string);

        if hotspot_enable_ids.contains(&tally_id) {
            info!("Enableing hotspot");
            let hotspot_enable_result = hotspot.lock().await.enable_hotspot().await;

            match hotspot_enable_result {
                Ok(_) => {
                    user_feedback
                        .lock()
                        .await
                        .set_device_status(feedback::DeviceStatus::HotspotEnabled);
                }
                Err(e) => {
                    error!("Hotspot: {e}");
                }
            }

            // TODO: Should the ID be added anyway or ignored ?
        }

        if id_store.lock().await.add_id(tally_id) {
            info!("Added new id to current day");

            user_feedback.lock().await.success().await;

            if let Err(e) = id_store.lock().await.export_json(STORE_PATH).await {
                error!("Failed to save id store to file: {e}");
                user_feedback.lock().await.failure().await;
                // TODO: How to handle a failure to save ?
            }
        }
    }

    Ok(())
}

async fn enter_error_state(feedback: Arc<Mutex<FeedbackImpl>>, hotspot: Arc<Mutex<impl Hotspot>>) {
    let _ = feedback.lock().await.activate_error_state().await;
    let _ = hotspot.lock().await.enable_hotspot().await;

    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    sigterm.recv().await;
}

#[tokio::main]
async fn main() -> Result<()> {
    logger::setup_logger();

    info!("Starting application");

    let user_feedback = Arc::new(Mutex::new(Feedback::new()?));
    let hotspot = Arc::new(Mutex::new(create_hotspot()?));

    let error_flag_set = args().any(|e| e == "--error" || e == "-e");
    if error_flag_set {
        error!("Error flag set. Entering error state");
        enter_error_state(user_feedback.clone(), hotspot).await;
        return Ok(());
    }

    let store: Arc<Mutex<IDStore>> = Arc::new(Mutex::new(load_or_create_store().await?));
    let hotspot_enable_ids = get_hotspot_enable_ids();

    let (tx, rx) = broadcast::channel::<String>(32);
    let sse_tx = tx.clone();

    let pm3_handle = run_pm3(tx);

    user_feedback.lock().await.startup().await;

    let loop_handle = handle_ids_loop(
        rx,
        hotspot_enable_ids,
        store.clone(),
        hotspot.clone(),
        user_feedback.clone(),
    );

    let webserver_handle = run_webserver(
        store.clone(),
        sse_tx,
        hotspot.clone(),
        user_feedback.clone(),
    );

    let run_result = try_join!(pm3_handle, loop_handle, webserver_handle);

    if let Err(e) = run_result {
        error!("Failed to run application: {e}");
        return Err(e);
    }

    Ok(())
}
