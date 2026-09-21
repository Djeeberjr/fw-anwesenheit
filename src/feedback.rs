use embassy_time::{Duration, Timer};
use esp_hal::gpio::Output;
use esp_hal_smartled::{LedAdapterError, SmartLedsAdapterAsync};
use log::{debug, error};
use smart_leds::SmartLedsWriteAsync;
use smart_leds::brightness;
use smart_leds::colors::{BLACK, GREEN, RED, YELLOW};

use crate::init::hardware;
use crate::{FEEDBACK_STATE, init};

#[derive(Copy, Clone, Debug)]
pub enum FeedbackState {
    Ack,
    Nack,
    Error,
    Startup,
    SilentAck,
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
        if let Err(err) = display_state(feedback_state, &mut led, &mut buzzer).await {
            error!("Failed to displat feedback state: {:?}", err);
        }
    }
}

async fn led_set(
    led: &mut SmartLedsAdapterAsync<'static, { hardware::LED_BUFFER_SIZE }>,
    color: smart_leds::RGB8,
) -> Result<(), LedAdapterError> {
    led.write(brightness(
        [color; init::hardware::NUM_LEDS].into_iter(),
        LED_LEVEL,
    ))
    .await
}

async fn display_state(
    state: FeedbackState,
    led: &mut SmartLedsAdapterAsync<'static, { hardware::LED_BUFFER_SIZE }>,
    buzzer: &mut Output<'static>,
) -> Result<(), LedAdapterError> {
    match state {
        FeedbackState::Ack => {
            led_set(led, GREEN).await?;
            buzzer.set_high();
            Timer::after(Duration::from_millis(100)).await;
            buzzer.set_low();
            Timer::after(Duration::from_millis(50)).await;
            led_set(led, BLACK).await?;
        }
        FeedbackState::Nack => {
            led_set(led, YELLOW).await?;
            buzzer.set_high();
            Timer::after(Duration::from_millis(100)).await;
            buzzer.set_low();
            Timer::after(Duration::from_millis(100)).await;
            buzzer.set_high();
            Timer::after(Duration::from_millis(100)).await;
            buzzer.set_low();
            led_set(led, BLACK).await?;
        }
        FeedbackState::Error => {
            led_set(led, RED).await?;
            buzzer.set_high();
            Timer::after(Duration::from_millis(500)).await;
            buzzer.set_low();
            Timer::after(Duration::from_millis(500)).await;
            buzzer.set_high();
            Timer::after(Duration::from_millis(500)).await;
            buzzer.set_low();
        }
        FeedbackState::Startup => {
            led_set(led, GREEN).await?;
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
            led_set(led, BLACK).await?;
        }
        FeedbackState::SilentAck => {
            led_set(led, GREEN).await?;
            Timer::after(Duration::from_millis(150)).await;
            led_set(led, BLACK).await?;
        }
    };

    Ok(())
}
