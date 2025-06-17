use anyhow::{Result, anyhow};
use log::{trace, warn};
use smart_leds::colors::GREEN;
use std::env;
use tokio::process::Command;

use crate::{feedback::{self, FeedbackImpl}, hardware::Hotspot, spi_led};

const SSID: &str = "fwa";
const CON_NAME: &str = "fwa-hotspot";
const PASSWORD: &str = "a9LG2kUVrsRRVUo1";
const IPV4_ADDRES: &str = "192.168.4.1/24";

/// NetworkManager Hotspot
pub struct NMHotspot {
    ssid: String,
    con_name: String,
    password: String,
    ipv4: String,
}

impl NMHotspot {
    pub fn new_from_env() -> Result<Self> {
        let ssid = env::var("HOTSPOT_SSID").unwrap_or(SSID.to_owned());
        let password = env::var("HOTSPOT_PW").unwrap_or_else(|_| {
            warn!("HOTSPOT_PW not set. Using default password");
            PASSWORD.to_owned()
        });

        if password.len() < 8 {
            return Err(anyhow!("Hotspot password to short"));
        }

        Ok(NMHotspot {
            ssid,
            con_name: CON_NAME.to_owned(),
            password,
            ipv4: IPV4_ADDRES.to_owned(),
        })
    }

    async fn create_hotspot(&self) -> Result<()> {
        let cmd = Command::new("nmcli")
            .args(["device", "wifi", "hotspot"])
            .arg("con-name")
            .arg(&self.con_name)
            .arg("ssid")
            .arg(&self.ssid)
            .arg("password")
            .arg(&self.password)
            .output()
            .await?;

        trace!("nmcli (std): {}", String::from_utf8_lossy(&cmd.stdout));
        trace!("nmcli (err): {}", String::from_utf8_lossy(&cmd.stderr));

        if !cmd.status.success() {
            return Err(anyhow!("nmcli command had non-zero exit code"));
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
            .await?;

        if !cmd.status.success() {
            return Err(anyhow!("nmcli command had non-zero exit code"));
        }

        Ok(())
    }

    /// Checks if the connection already exists
    async fn exists(&self) -> Result<bool> {
        let cmd = Command::new("nmcli")
            .args(["connection", "show"])
            .arg(&self.con_name)
            .output()
            .await?;

        trace!("nmcli (std): {}", String::from_utf8_lossy(&cmd.stdout));
        trace!("nmcli (err): {}", String::from_utf8_lossy(&cmd.stderr));

        Ok(cmd.status.success())
    }
}

impl Hotspot for NMHotspot {
    async fn enable_hotspot(&self) -> Result<()> {
        if !self.exists().await? {
            self.create_hotspot().await?;
        }

        let cmd = Command::new("nmcli")
            .args(["connection", "up"])
            .arg(&self.con_name)
            .output()
            .await?;

        trace!("nmcli (std): {}", String::from_utf8_lossy(&cmd.stdout));
        trace!("nmcli (err): {}", String::from_utf8_lossy(&cmd.stderr));

        if !cmd.status.success() {
            return Err(anyhow!("nmcli command had non-zero exit code"));
        }

        Ok(())
    }

    async fn disable_hotspot(&self) -> Result<()> {
        let cmd = Command::new("nmcli")
            .args(["connection", "down"])
            .arg(&self.con_name)
            .output()
            .await?;

        trace!("nmcli (std): {}", String::from_utf8_lossy(&cmd.stdout));
        trace!("nmcli (err): {}", String::from_utf8_lossy(&cmd.stderr));

        if !cmd.status.success() {
            return Err(anyhow!("nmcli command had non-zero exit code"));
        }

        feedback::CURRENTSTATUS = Ready;
        FeedbackImpl::flash_led_for_duration(led, GREEN, 1000);
        
        Ok(())
    }
}
