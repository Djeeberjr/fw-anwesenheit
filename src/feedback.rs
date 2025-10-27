use embassy_time::{Duration, Timer};
use esp_hal::gpio::{AnyPin, Output};
use esp_hal::peripherals;
use esp_hal::twai::TimingConfig;
use esp_hal_smartled::SmartLedsAdapterAsync;
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
    mut led: SmartLedsAdapterAsync<'static, { hardware::LED_BUFFER_SIZE }>,
    mut buzzer: Output<'static>,
) {
    debug!("Starting feedback task");
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

                Timer::after(Duration::from_secs(2)).await;
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
