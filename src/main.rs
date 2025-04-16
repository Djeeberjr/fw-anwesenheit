use pm3::{pm3_mock, run_pm3};
use tokio::sync::mpsc;
use webserver::start_webserver;

mod id_store;
mod parser;
mod pm3;
mod webserver;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(32);

    tokio::spawn(async move {
        match pm3_mock(tx).await {
            Ok(()) => {
                println!("PM3 exited with an zero error code");
            }
            Err(e) => {
                println!("PM3 failed to run: {}", e);
            }
        }
    });

    tokio::spawn(async move {
        while let Some(line) = rx.recv().await {
            println!("Got from channel: {}", line);
        }
    });

    match start_webserver().await {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Failed to start webserver: {}", e);
        }
    }
}
