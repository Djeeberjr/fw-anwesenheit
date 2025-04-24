use std::sync::Arc;

use id_store::IDStore;
use pm3::{pm3_mock, run_pm3};
use tokio::sync::{mpsc, Mutex};
use webserver::start_webserver;

mod id_store;
mod parser;
mod pm3;
mod webserver;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(1);

    tokio::spawn(async move {
        match run_pm3(tx).await {
            Ok(()) => {
                println!("PM3 exited with an zero error code");
            }
            Err(e) => {
                println!("PM3 failed to run: {}", e);
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
            eprintln!("Failed to start webserver: {}", e);
        }
    }
}
