use log::error;
use rgb::RGB8;
use smart_leds::colors::{GREEN, RED};
use std::{error::Error, time::Duration};
use tokio::{join, time::sleep};

use crate::hardware::{Buzzer, StatusLed};

#[cfg(not(feature = "mock_pi"))]
use crate::{gpio_buzzer::GPIOBuzzer, spi_led::SpiLed};

#[cfg(feature = "mock_pi")]
use crate::mock::{MockBuzzer, MockLed};

const LED_BLINK_DURATION: Duration = Duration::from_secs(1);

pub struct Feedback<B: Buzzer, L: StatusLed> {
    buzzer: B,
    led: L,
}

impl<B: Buzzer, L: StatusLed> Feedback<B, L> {
    pub async fn success(&mut self) {
        let buzzer_handle = Self::beep_ack(&mut self.buzzer);
        let led_handle = Self::blink_led_for_duration(&mut self.led, GREEN, LED_BLINK_DURATION);
        let (buzzer_result, _) = join!(buzzer_handle, led_handle);

        buzzer_result.unwrap_or_else(|err| {
            error!("Failed to buzz: {err}");
        });
    }

    pub async fn failure(&mut self) {
        let buzzer_handle = Self::beep_nak(&mut self.buzzer);
        let led_handle = Self::blink_led_for_duration(&mut self.led, RED, LED_BLINK_DURATION);

        let (buzzer_result, _) = join!(buzzer_handle, led_handle);

        buzzer_result.unwrap_or_else(|err| {
            error!("Failed to buzz: {err}");
        });
    }

    async fn blink_led_for_duration(
        led: &mut L,
        color: RGB8,
        duration: Duration,
    ) -> Result<(), Box<dyn Error>> {
        led.turn_on(color)?;
        sleep(duration).await;
        led.turn_off()?;
        Ok(())
    }

    async fn beep_ack(buzzer: &mut B) -> Result<(), Box<dyn Error>> {
        buzzer
            .modulated_tone(1200.0, Duration::from_millis(100))
            .await?;
        sleep(Duration::from_millis(10)).await;
        buzzer
            .modulated_tone(2000.0, Duration::from_millis(50))
            .await?;
        Ok(())
    }

    async fn beep_nak(buzzer: &mut B) -> Result<(), Box<dyn Error>> {
        buzzer
            .modulated_tone(600.0, Duration::from_millis(150))
            .await?;
        sleep(Duration::from_millis(100)).await;
        buzzer
            .modulated_tone(600.0, Duration::from_millis(150))
            .await?;
        Ok(())
    }
}

#[cfg(feature = "mock_pi")]
pub type FeedbackImpl = Feedback<MockBuzzer, MockLed>;
#[cfg(not(feature = "mock_pi"))]
pub type FeedbackImpl = Feedback<GPIOBuzzer, SpiLed>;

impl FeedbackImpl {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        #[cfg(feature = "mock_pi")]
        {
            Ok(Feedback {
                buzzer: MockBuzzer {},
                led: MockLed {},
            })
        }
        #[cfg(not(feature = "mock_pi"))]
        {
            Ok(Feedback {
                buzzer: GPIOBuzzer::new_default()?,
                led: SpiLed::new()?,
            })
        }
    }
}
