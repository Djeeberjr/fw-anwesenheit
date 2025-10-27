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

static ESP_WIFI_CTRL: StaticCell<Controller<'static>> = StaticCell::new();

pub async fn set_antenna_mode(gpio3: GPIO3<'static>, gpio14: GPIO14<'static>) {
    let mut rf_switch = Output::new(gpio3, esp_hal::gpio::Level::Low, OutputConfig::default());

    rf_switch.set_low();

    Timer::after_millis(150).await;

    let mut antenna_mode = Output::new(gpio14, esp_hal::gpio::Level::Low, OutputConfig::default());

    antenna_mode.set_low();
}

pub fn setup_wifi<'d: 'static>(wifi: WIFI<'static>, spawner: Spawner) -> Interfaces<'d> {
    let esp_wifi_ctrl = ESP_WIFI_CTRL.init(esp_radio::init().unwrap());

    let config = esp_radio::wifi::Config::default();
    let (controller, interfaces) = esp_radio::wifi::new(esp_wifi_ctrl, wifi, config).unwrap();

    spawner.must_spawn(connection(controller));

    interfaces
}
#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    debug!("start connection task");
    debug!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_radio::wifi::ap_state() {
            WifiApState::Started => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::ApStop).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
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
