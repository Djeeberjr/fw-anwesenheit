use log::{debug, info, trace, warn};
use std::env;
use std::error::Error;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
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
        .stdin(Stdio::null())
        .spawn()?;

    let stdout = cmd.stdout.take().ok_or("Failed to get stdout")?;

    let mut reader = BufReader::new(stdout).lines();

    while let Some(line) = reader.next_line().await? {
        trace!("PM3: {line}");
        if let Some(uid) = super::parser::parse_line(&line) {
            tx.send(uid)?;
        }
    }

    let status = cmd.wait().await?;

    if status.success() {
        Ok(())
    } else {
        Err("PM3 exited with a non zero exit code".into())
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
