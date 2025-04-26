use log::{debug, error, info};
use std::env;
use std::error::Error;
use std::io::{self, BufRead};
use std::process::{Command, Stdio};
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

pub async fn run_pm3(tx: mpsc::Sender<String>) -> Result<(), Box<dyn Error>> {
    let pm3_path = match env::var("PM3_BIN") {
        Ok(path) => path,
        Err(_) => {
            info!("PM3_BIN not set. Using default value");
            "pm3".to_owned()
        }
    };

    let mut cmd = Command::new("stdbuf")
        .arg("-oL")
        .arg(pm3_path)
        .arg("-c")
        .arg("lf hitag reader -@")
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = cmd.stdout.take().ok_or("Failed to get stdout")?;
    let reader = io::BufReader::new(stdout);

    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                debug!("PM3: {}", line);
                let parse_result = super::parser::parse_line(&line);
                if let Some(uid) = parse_result {
                    debug!("Read ID: {}", uid);
                    match tx.send(uid).await {
                        Ok(()) => {}
                        Err(e) => {
                            error!("Failed to send to channel: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to read line from PM3: {}", e);
            }
        }
    }

    let status = cmd.wait().expect("Failed to wait on child");

    if status.success() {
        Ok(())
    } else {
        Err("PM3 exited with a non zero exit code".into())
    }
}

pub async fn pm3_mock(tx: mpsc::Sender<String>) -> Result<(), Box<dyn Error>> {
    #![allow(while_true)]
    while true {
        match tx.send("F1409618".to_owned()).await {
            Ok(()) => {}
            Err(e) => {
                error!("Failed to send to channel: {}", e);
            }
        }

        sleep(Duration::from_millis(1000)).await;
    }

    Ok(())
}
