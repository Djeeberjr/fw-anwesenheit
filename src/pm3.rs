use log::{debug, info, trace, warn};
use std::env;
use std::error::Error;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::select;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::broadcast;

/// Runs the pm3 binary and monitors it's output
/// The pm3 binary is ether set in the env var PM3_BIN or found in the path
/// The ouput is parsed and send via the `tx` channel
pub async fn run_pm3(tx: broadcast::Sender<String>) -> Result<(), Box<dyn Error>> {
    kill_orphans().await;

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
        .stderr(Stdio::null())
        .stdin(Stdio::piped())
        .spawn()?;

    let stdout = cmd.stdout.take().ok_or("Failed to get stdout")?;
    let mut stdin = cmd.stdin.take().ok_or("Failed to get stdin")?;

    let mut reader = BufReader::new(stdout).lines();

    let mut sigterm = signal(SignalKind::terminate())?;

    let child_handle = tokio::spawn(async move {
        let mut last_id: String = "".to_owned();
        while let Some(line) = reader.next_line().await.unwrap_or(None) {
            trace!("PM3: {line}");
            if let Some(uid) = super::parser::parse_line(&line) {
                if last_id == uid {
                    let _ = tx.send(uid.clone());
                }
                last_id = uid;
            }
        }
    });

    select! {
        _ = child_handle => {}
        _ = sigterm.recv() => {
            debug!("Graceful shutdown of PM3");
            let _ = stdin.write_all(b"\n").await;
            let _ = stdin.flush().await;
        }
    };

    let status = cmd.wait().await?;
    if status.success() {
        Ok(())
    } else {
        Err("PM3 exited with a non-zero exit code".into())
    }
}

/// Kills any open pm3 instances
/// Also funny name. hehehe.
async fn kill_orphans() {
    let kill_result = Command::new("pkill")
        .arg("-KILL")
        .arg("-x")
        .arg("proxmark3")
        .output()
        .await;

    match kill_result {
        Ok(_) => {
            debug!("Successfully killed orphaned pm3 instances");
        }
        Err(e) => {
            warn!("Failed to kill pm3 orphans: {e} Continuing anyway");
        }
    }
}
