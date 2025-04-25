use std::error::Error;
use tokio::process::Command;

const SSID: &str = "fwa";
const CON_NAME: &str = "fwa-hotspot";
const PASSWORD: &str = "hunter22";
const IPV4_ADDRES: &str = "192.168.4.1/24";

async fn create_hotspot() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::new("nmcli")
        .args(["device", "wifi", "hotspot"])
        .arg("con-name")
        .arg(CON_NAME)
        .arg("ssid")
        .arg(SSID)
        .arg("password")
        .arg(PASSWORD)
        .spawn()?;

    let status = cmd.wait().await?;

    if !status.success() {
        return Err("Failed to create hotspot".into());
    }

    let mut cmd = Command::new("nmcli")
        .arg("connection")
        .arg("modify")
        .arg(CON_NAME)
        .arg("ipv4.method")
        .arg("shared")
        .arg("ipv4.addresses")
        .arg(IPV4_ADDRES)
        .spawn()?;

    let status = cmd.wait().await?;

    if !status.success() {
        return Err("Failed to create hotspot".into());
    }

    Ok(())
}

async fn exists() -> Result<bool, Box<dyn Error>> {
    let mut cmd = Command::new("nmcli")
        .args(["connection", "show"])
        .arg(CON_NAME)
        .spawn()?;

    let status = cmd.wait().await?;

    Ok(status.success())
}

pub async fn enable_hotspot() -> Result<(), Box<dyn Error>> {
    if !exists().await? {
        create_hotspot().await?;
    }

    let mut cmd = Command::new("nmcli")
        .args(["connection", "up"])
        .arg(CON_NAME)
        .spawn()?;

    let status = cmd.wait().await?;

    if !status.success() {
        return Err("Failed to enable hotspot".into());
    }

    Ok(())
}

pub async fn disable_hotspot() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::new("nmcli")
        .args(["connection", "down"])
        .arg(CON_NAME)
        .spawn()?;

    let status = cmd.wait().await?;

    if !status.success() {
        return Err("Failed to enable hotspot".into());
    }

    Ok(())
}
