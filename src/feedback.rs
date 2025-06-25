use anyhow::Result;
use log::error;
use rgb::RGB8;
use smart_leds::colors::{GREEN, RED};
use std::time::Duration;
use tokio::{join, time::sleep};

use crate::hardware::{Buzzer, StatusLed};

#[cfg(not(feature = "mock_pi"))]
use crate::{gpio_buzzer::GPIOBuzzer, spi_led::SpiLed};

#[cfg(feature = "mock_pi")]
use crate::mock::{MockBuzzer, MockLed};

const LED_BLINK_DURATION: Duration = Duration::from_secs(1);

pub enum DeviceStatus {
    NotReady,
    Ready,
    HotspotEnabled,
}

impl DeviceStatus {
    pub fn color(&self) -> RGB8 {
        match self {
            Self::NotReady => RGB8::new(0, 0, 0),
            Self::Ready => RGB8::new(0, 50, 0),
            Self::HotspotEnabled => RGB8::new(0, 0, 50),
        }
    }
}
pub struct Feedback<B: Buzzer, L: StatusLed> {
    device_status: DeviceStatus,
    buzzer: B,
    led: L,
}

impl<B: Buzzer, L: StatusLed> Feedback<B, L> {
    pub async fn success(&mut self) {
        let buzzer_handle = Self::beep_ack(&mut self.buzzer);
        let led_handle = Self::flash_led_for_duration(&mut self.led, GREEN, LED_BLINK_DURATION);
        let (buzzer_result, _) = join!(buzzer_handle, led_handle);

        buzzer_result.unwrap_or_else(|err| {
            error!("Failed to buzz: {err}");
        });

        let _ = self.led_to_status();
    }

    pub async fn failure(&mut self) {
        let buzzer_handle = Self::beep_nak(&mut self.buzzer);
        let led_handle = Self::flash_led_for_duration(&mut self.led, RED, LED_BLINK_DURATION);

        let (buzzer_result, _) = join!(buzzer_handle, led_handle);

        buzzer_result.unwrap_or_else(|err| {
            error!("Failed to buzz: {err}");
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

    pub fn set_device_status(&mut self, status: DeviceStatus){
        self.device_status = status;
        let _ = self.led_to_status();
    }

    fn led_to_status(&mut self) -> Result<()> {
        self.led.turn_on(self.device_status.color())
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
}

#[cfg(feature = "mock_pi")]
pub type FeedbackImpl = Feedback<MockBuzzer, MockLed>;
#[cfg(not(feature = "mock_pi"))]
pub type FeedbackImpl = Feedback<GPIOBuzzer, SpiLed>;

impl FeedbackImpl {
    pub fn new() -> Result<Self> {
        #[cfg(feature = "mock_pi")]
        {
            Ok(Feedback {
                device_status: DeviceStatus::NotReady,
                buzzer: MockBuzzer {},
                led: MockLed {},
            })
        }
        #[cfg(not(feature = "mock_pi"))]
        {
            Ok(Feedback {
                device_status: DeviceStatus::NotReady,
                buzzer: GPIOBuzzer::new_default()?,
                led: SpiLed::new()?,
            })
        }
    }
}
