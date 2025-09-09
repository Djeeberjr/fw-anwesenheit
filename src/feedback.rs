use embassy_time::{Delay, Duration, Timer};
use esp_hal::{delay, gpio::Output, peripherals, rmt::ConstChannelAccess};
use esp_hal_smartled::SmartLedsAdapterAsync;
use init::hardware;
use log::{debug, error, info};
use smart_leds::SmartLedsWriteAsync;
use smart_leds::colors::{BLACK, GREEN, RED, YELLOW};
use smart_leds::{brightness, colors::BLUE};

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
    mut led: SmartLedsAdapterAsync<
        ConstChannelAccess<esp_hal::rmt::Tx, 0>,
        { init::hardware::LED_BUFFER_SIZE },
    >,
    buzzer: peripherals::GPIO21<'static>,
) {
    debug!("Starting feedback task");
    let mut buzzer = init::hardware::setup_buzzer(buzzer);
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
                    [GREEN; init::hardware::NUM_LEDS].into_iter(),
                    LED_LEVEL,
                ))
                .await
                .unwrap();
            }
        };
        debug!("Feedback state: {:?}", feedback_state);
    }
}

// async fn beep_ack() {
//     buzzer.set_high();
//     buzzer.set_low();
//     //Timer::after(Duration::from_millis(100)).await;
// }

/* pub async fn failure(&mut self) {
    let buzzer_handle = Self::beep_nak(&mut self.buzzer);
    let led_handle = Self::flash_led_for_duration(&mut self.led, RED, LED_BLINK_DURATION);

    let (buzzer_result, _) = join!(buzzer_handle, led_handle);

    buzzer_result.unwrap_or_else(|err| {            error!("Failed to buzz: {err}");
    });

    let _ = self.led_to_status();
}

pub async fn activate_error_state(&mut self) -> Result<()> {
    self.led.turn_on(RED)?;
    Self::beep_nak(&mut self.buzzer).await?;
    Ok(())
}

pub async fn startup(&mut self){
    self.device_status = DeviceStatus::Ready;

    let led_handle = Self::flash_led_for_duration(&mut self.led, GREEN, Duration::from_secs(1));
    let buzzer_handle = Self::beep_startup(&mut self.buzzer);

    let (buzzer_result, led_result) = join!(buzzer_handle, led_handle);

    buzzer_result.unwrap_or_else(|err| {
        error!("Failed to buzz: {err}");
    });

    led_result.unwrap_or_else(|err| {
        error!("Failed to blink led: {err}");
    });

    let _ = self.led_to_status();
}


async fn flash_led_for_duration(led: &mut L, color: RGB8, duration: Duration) -> Result<()> {
    led.turn_on(color)?;

    sleep(duration).await;

    led.turn_off()?;

    Ok(())
}

async fn beep_ack(buzzer: &mut B) -> Result<()> {
    buzzer
        .modulated_tone(1200.0, Duration::from_millis(100))
        .await?;
    sleep(Duration::from_millis(10)).await;
    buzzer
        .modulated_tone(2000.0, Duration::from_millis(50))
        .await?;
    Ok(())
}

async fn beep_nak(buzzer: &mut B) -> Result<()> {
    buzzer
        .modulated_tone(600.0, Duration::from_millis(150))
        .await?;
    sleep(Duration::from_millis(100)).await;
    buzzer
        .modulated_tone(600.0, Duration::from_millis(150))
        .await?;
    Ok(())
}

async fn beep_startup(buzzer: &mut B) -> Result<()> {
    buzzer
        .modulated_tone(523.0, Duration::from_millis(150))
        .await?;
    buzzer
        .modulated_tone(659.0, Duration::from_millis(150))
        .await?;
    buzzer
        .modulated_tone(784.0, Duration::from_millis(150))
        .await?;
    buzzer
        .modulated_tone(1046.0, Duration::from_millis(200))
        .await?;

    sleep(Duration::from_millis(100)).await;

    buzzer
        .modulated_tone(784.0, Duration::from_millis(100))
        .await?;
    buzzer
        .modulated_tone(880.0, Duration::from_millis(200))
        .await?;
    Ok(())
}

 */
