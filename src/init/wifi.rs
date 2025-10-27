use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Output, OutputConfig};
use esp_hal::peripherals::{GPIO3, GPIO14, WIFI};
use esp_radio::Controller;
use esp_radio::wifi::{
    AccessPointConfig, Interfaces, ModeConfig, WifiApState, WifiController, WifiEvent,
};
use log::debug;
use static_cell::StaticCell;
use thiserror::Error;

pub async fn set_antenna_mode(gpio3: GPIO3<'static>, gpio14: GPIO14<'static>) {
    let mut rf_switch = Output::new(gpio3, esp_hal::gpio::Level::Low, OutputConfig::default());

    rf_switch.set_low();

    Timer::after_millis(150).await;

    let mut antenna_mode = Output::new(gpio14, esp_hal::gpio::Level::Low, OutputConfig::default());

    antenna_mode.set_low();
}

#[derive(Error, Debug)]
pub enum WifiError {
    #[error("Failed to init radio")]
    Init(#[from] esp_radio::InitializationError),

    #[error("Failed to init wifi")]
    Wifi(#[from] esp_radio::wifi::WifiError),

    #[error("Failed to spawn wifi task")]
    Spawn(#[from] embassy_executor::SpawnError),
}

pub fn setup_wifi<'d: 'static>(
    wifi: WIFI<'static>,
    spawner: Spawner,
) -> Result<Interfaces<'d>, WifiError> {
    static ESP_WIFI_CTRL: StaticCell<Controller<'static>> = StaticCell::new();

    let esp_wifi_ctrl = ESP_WIFI_CTRL.init(esp_radio::init()?);

    let config = esp_radio::wifi::Config::default();
    let (controller, interfaces) = esp_radio::wifi::new(esp_wifi_ctrl, wifi, config)?;

    spawner.spawn(connection(controller))?;

    Ok(interfaces)
}
#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    debug!("start connection task");
    debug!("Device capabilities: {:?}", controller.capabilities());

    loop {
        if esp_radio::wifi::ap_state() == WifiApState::Started {
            // wait until we're no longer connected
            controller.wait_for_event(WifiEvent::ApStop).await;
            Timer::after(Duration::from_millis(5000)).await
        }

        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = ModeConfig::AccessPoint(
                AccessPointConfig::default()
                    .with_ssid(env!("WIFI_SSID").try_into().unwrap())
                    .with_password(env!("WIFI_PASSWD").try_into().unwrap())
                    .with_auth_method(esp_radio::wifi::AuthMethod::Wpa2Personal),
            );
            controller.set_config(&client_config).unwrap();
            debug!("Starting wifi");
            controller.start_async().await.unwrap();
            debug!("Wifi started!");
        }
    }
}
