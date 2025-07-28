use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Output, OutputConfig};
use esp_hal::peripherals::{GPIO3, GPIO14, WIFI};
use esp_wifi::wifi::{AccessPointConfiguration, Configuration, WifiController, WifiEvent, WifiState};
use esp_wifi::{EspWifiRngSource, EspWifiTimerSource, wifi::Interfaces};
use static_cell::make_static;

pub async fn set_antenna_mode(gpio3: GPIO3<'static>, gpio14: GPIO14<'static>) {
    let mut rf_switch = Output::new(gpio3, esp_hal::gpio::Level::Low, OutputConfig::default());

    rf_switch.set_low();

    Timer::after_millis(150).await;

    let mut antenna_mode = Output::new(gpio14, esp_hal::gpio::Level::Low, OutputConfig::default());

    antenna_mode.set_low();
}

pub fn setup_wifi<'d: 'static>(
    timer: impl EspWifiTimerSource + 'd,
    rng: impl EspWifiRngSource + 'd,
    wifi: WIFI<'static>,
    spawner: &mut Spawner,
) -> Interfaces<'d> {
    let esp_wifi_ctrl = make_static!(esp_wifi::init(timer, rng).unwrap());

    let (controller, interfaces) = esp_wifi::wifi::new(esp_wifi_ctrl, wifi).unwrap();

    spawner.must_spawn(connection(controller));

    interfaces
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::ApStarted => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::ApStop).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::AccessPoint(AccessPointConfiguration {
                ssid: "esp-wifi".try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            controller.start_async().await.unwrap();
        }
    }
}
