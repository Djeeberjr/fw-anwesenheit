use log::trace;
use std::{
    fmt::{self},
    process::Output,
};
use tokio::process::Command;

const SSID: &str = "fwa";
const CON_NAME: &str = "fwa-hotspot";
const PASSWORD: &str = "hunter22";
const IPV4_ADDRES: &str = "192.168.4.1/24";

#[derive(Debug)]
pub enum HotspotError {
    IoError(std::io::Error),
    NonZeroExit(Output),
}

impl fmt::Display for HotspotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HotspotError::IoError(err) => {
                write!(f, "Failed to run hotspot command. I/O error: {err}")
            }
            HotspotError::NonZeroExit(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                write!(
                    f,
                    "Failed to run hotspot command.\nStdout: {stdout}\nStderr: {stderr}",
                )
            }
        }
    }
}

impl std::error::Error for HotspotError {}

/// Create the connection in NM
/// Will fail if already exists
async fn create_hotspot() -> Result<(), HotspotError> {
    let cmd = Command::new("nmcli")
        .args(["device", "wifi", "hotspot"])
        .arg("con-name")
        .arg(CON_NAME)
        .arg("ssid")
        .arg(SSID)
        .arg("password")
        .arg(PASSWORD)
        .output()
        .await
        .map_err(HotspotError::IoError)?;

    trace!("nmcli (std): {}", String::from_utf8_lossy(&cmd.stdout));
    trace!("nmcli (err): {}", String::from_utf8_lossy(&cmd.stderr));

    if !cmd.status.success() {
        return Err(HotspotError::NonZeroExit(cmd));
    }

    let cmd = Command::new("nmcli")
        .arg("connection")
        .arg("modify")
        .arg(CON_NAME)
        .arg("ipv4.method")
        .arg("shared")
        .arg("ipv4.addresses")
        .arg(IPV4_ADDRES)
        .output()
        .await
        .map_err(HotspotError::IoError)?;

    if !cmd.status.success() {
        return Err(HotspotError::NonZeroExit(cmd));
    }

    Ok(())
}

/// Checks if the connection already exists
async fn exists() -> Result<bool, HotspotError> {
    let cmd = Command::new("nmcli")
        .args(["connection", "show"])
        .arg(CON_NAME)
        .output()
        .await
        .map_err(HotspotError::IoError)?;

    trace!("nmcli (std): {}", String::from_utf8_lossy(&cmd.stdout));
    trace!("nmcli (err): {}", String::from_utf8_lossy(&cmd.stderr));

    Ok(cmd.status.success())
}

pub async fn enable_hotspot() -> Result<(), HotspotError> {
    if !exists().await? {
        create_hotspot().await?;
    }

    let cmd = Command::new("nmcli")
        .args(["connection", "up"])
        .arg(CON_NAME)
        .output()
        .await
        .map_err(HotspotError::IoError)?;

    trace!("nmcli (std): {}", String::from_utf8_lossy(&cmd.stdout));
    trace!("nmcli (err): {}", String::from_utf8_lossy(&cmd.stderr));

    if !cmd.status.success() {
        return Err(HotspotError::NonZeroExit(cmd));
    }

    Ok(())
}

pub async fn disable_hotspot() -> Result<(), HotspotError> {
    let cmd = Command::new("nmcli")
        .args(["connection", "down"])
        .arg(CON_NAME)
        .output()
        .await
        .map_err(HotspotError::IoError)?;

    trace!("nmcli (std): {}", String::from_utf8_lossy(&cmd.stdout));
    trace!("nmcli (err): {}", String::from_utf8_lossy(&cmd.stderr));

    if !cmd.status.success() {
        return Err(HotspotError::NonZeroExit(cmd));
    }

    Ok(())
}
