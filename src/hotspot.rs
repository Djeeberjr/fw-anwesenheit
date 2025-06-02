use log::{error, trace, warn};
use std::{
    env,
    fmt::{self},
    process::Output,
};
use tokio::process::Command;

use crate::hardware::Hotspot;

const SSID: &str = "fwa";
const CON_NAME: &str = "fwa-hotspot";
const PASSWORD: &str = "a9LG2kUVrsRRVUo1";
const IPV4_ADDRES: &str = "192.168.4.1/24";

#[derive(Debug)]
pub enum HotspotError {
    IoError(std::io::Error),
    NonZeroExit(Output),
    PasswordToShort,
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
            HotspotError::PasswordToShort => {
                write!(f, "The password must be at leat 8 characters long")
            }
        }
    }
}

impl std::error::Error for HotspotError {}

/// NetworkManager Hotspot
pub struct NMHotspot {
    ssid: String,
    con_name: String,
    password: String,
    ipv4: String,
}

impl NMHotspot {
    pub fn new_from_env() -> Result<Self, HotspotError> {
        let ssid = env::var("HOTSPOT_SSID").unwrap_or(SSID.to_owned());
        let password = env::var("HOTSPOT_PW").unwrap_or_else(|_| {
            warn!("HOTSPOT_PW not set. Using default password");
            PASSWORD.to_owned()
        });

        if password.len() < 8 {
            error!("Hotspot PW is to short");
            return Err(HotspotError::PasswordToShort);
        }

        Ok(NMHotspot {
            ssid,
            con_name: CON_NAME.to_owned(),
            password,
            ipv4: IPV4_ADDRES.to_owned(),
        })
    }

    async fn create_hotspot(&self) -> Result<(), HotspotError> {
        let cmd = Command::new("nmcli")
            .args(["device", "wifi", "hotspot"])
            .arg("con-name")
            .arg(&self.con_name)
            .arg("ssid")
            .arg(&self.ssid)
            .arg("password")
            .arg(&self.password)
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
            .arg(&self.con_name)
            .arg("ipv4.method")
            .arg("shared")
            .arg("ipv4.addresses")
            .arg(&self.ipv4)
            .output()
            .await
            .map_err(HotspotError::IoError)?;

        if !cmd.status.success() {
            return Err(HotspotError::NonZeroExit(cmd));
        }

        Ok(())
    }

    /// Checks if the connection already exists
    async fn exists(&self) -> Result<bool, HotspotError> {
        let cmd = Command::new("nmcli")
            .args(["connection", "show"])
            .arg(&self.con_name)
            .output()
            .await
            .map_err(HotspotError::IoError)?;

        trace!("nmcli (std): {}", String::from_utf8_lossy(&cmd.stdout));
        trace!("nmcli (err): {}", String::from_utf8_lossy(&cmd.stderr));

        Ok(cmd.status.success())
    }
}

impl Hotspot for NMHotspot {
    async fn enable_hotspot(&self) -> Result<(), HotspotError> {
        if !self.exists().await? {
            self.create_hotspot().await?;
        }

        let cmd = Command::new("nmcli")
            .args(["connection", "up"])
            .arg(&self.con_name)
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

    async fn disable_hotspot(&self) -> Result<(), HotspotError> {
        let cmd = Command::new("nmcli")
            .args(["connection", "down"])
            .arg(&self.con_name)
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
}
