use embassy_time::{Duration, Timer};
use esp_hal::peripherals::RMT;
use esp_hal::rmt::Rmt;
use esp_hal::time::Rate;
use esp_hal::{peripherals, rmt};
use esp_hal_smartled::{SmartLedsAdapterAsync, buffer_size_async};
use log::debug;
use smart_leds::SmartLedsWriteAsync;
use smart_leds::colors::{BLACK, GREEN, RED, YELLOW};
use smart_leds::{brightness, colors::BLUE};

use crate::init::hardware;
use crate::{FEEDBACK_STATE, init};

#[derive(Copy, Clone, Debug)]
pub enum FeedbackState {
    Ack,
    Nack,
    Error,
    Startup,
    WIFI,
    Idle,
}

const LED_LEVEL: u8 = 255;

//TODO ERROR STATE: 1 Blink = unknows error,  3 Blink = no sd card

#[embassy_executor::task]
pub async fn feedback_task(
    rmt: Rmt<'static, esp_hal::Async>,
    led_gpio: peripherals::GPIO1<'static>,
    buzzer_gpio: peripherals::GPIO21<'static>,
) {
    debug!("Starting feedback task");

    let rmt_channel = rmt.channel0;
    let rmt_buffer = [esp_hal::rmt::PulseCode::default(); buffer_size_async(hardware::NUM_LEDS)];

    let mut led = SmartLedsAdapterAsync::new(rmt_channel, led_gpio, rmt_buffer);

    let mut buzzer = init::hardware::setup_buzzer(buzzer_gpio);
    loop {
        let feedback_state = FEEDBACK_STATE.wait().await;
        match feedback_state {
            FeedbackState::Ack => {
                led.write(brightness(
                    [GREEN; init::hardware::NUM_LEDS].into_iter(),
                    LED_LEVEL,
                ))
                .await
                .unwrap();
                buzzer.set_high();
                Timer::after(Duration::from_millis(100)).await;
                buzzer.set_low();
                Timer::after(Duration::from_millis(50)).await;
                led.write(brightness(
                    [BLACK; init::hardware::NUM_LEDS].into_iter(),
                    LED_LEVEL,
                ))
                .await
                .unwrap();
            }
            FeedbackState::Nack => {
                led.write(brightness(
                    [YELLOW; init::hardware::NUM_LEDS].into_iter(),
                    LED_LEVEL,
                ))
                .await
                .unwrap();
                buzzer.set_high();
                Timer::after(Duration::from_millis(100)).await;
                buzzer.set_low();
                Timer::after(Duration::from_millis(100)).await;
                buzzer.set_high();
                Timer::after(Duration::from_millis(100)).await;
                buzzer.set_low();
                led.write(brightness(
                    [BLACK; init::hardware::NUM_LEDS].into_iter(),
                    LED_LEVEL,
                ))
                .await
                .unwrap();
            }
            FeedbackState::Error => {
                led.write(brightness(
                    [RED; init::hardware::NUM_LEDS].into_iter(),
                    LED_LEVEL,
                ))
                .await
                .unwrap();
                buzzer.set_high();
                Timer::after(Duration::from_millis(500)).await;
                buzzer.set_low();
                Timer::after(Duration::from_millis(500)).await;
                buzzer.set_high();
                Timer::after(Duration::from_millis(500)).await;
                buzzer.set_low();
            }
            FeedbackState::Startup => {
                led.write(brightness(
                    [GREEN; init::hardware::NUM_LEDS].into_iter(),
                    LED_LEVEL,
                ))
                .await
                .unwrap();
                buzzer.set_high();
                Timer::after(Duration::from_millis(10)).await;
                buzzer.set_low();
                Timer::after(Duration::from_millis(10)).await;
                buzzer.set_high();
                Timer::after(Duration::from_millis(10)).await;
                buzzer.set_low();
                Timer::after(Duration::from_millis(50)).await;
                buzzer.set_high();
                Timer::after(Duration::from_millis(100)).await;
                buzzer.set_low();
                led.write(brightness(
                    [BLACK; init::hardware::NUM_LEDS].into_iter(),
                    LED_LEVEL,
                ))
                .await
                .unwrap();
            }
            FeedbackState::WIFI => {
                led.write(brightness(
                    [BLUE; init::hardware::NUM_LEDS].into_iter(),
                    LED_LEVEL,
                ))
                .await
                .unwrap();
            }
            FeedbackState::Idle => {
                led.write(brightness(
                    [BLACK; init::hardware::NUM_LEDS].into_iter(),
                    LED_LEVEL,
                ))
                .await
                .unwrap();
            }
        };
        debug!("Feedback state: {:?}", feedback_state);
    }
}
