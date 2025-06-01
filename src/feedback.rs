use log::error;
use rppal::pwm::Channel;
use std::error::Error;
use tokio::join;

use crate::{
    buzzer::{Buzzer, GPIOBuzzer},
    led::{SpiLed, StatusLed},
};

#[cfg(feature = "mock_pi")]
use crate::mock::{MockBuzzer, MockLed};

const PWM_CHANNEL_BUZZER: Channel = Channel::Pwm0; //PWM0 = GPIO18/Physical pin 12

pub struct Feedback<B: Buzzer, L: StatusLed> {
    buzzer: B,
    led: L,
}

impl<B: Buzzer, L: StatusLed> Feedback<B, L> {
    pub async fn success(&mut self) {
        let (buzzer_result, led_result) =
            join!(self.buzzer.beep_ack(), self.led.turn_green_on_1s());

        buzzer_result.unwrap_or_else(|err| {
            error!("Failed to buzz: {err}");
        });

        led_result.unwrap_or_else(|err| {
            error!("Failed to set LED: {err}");
        });
    }

    pub async fn failure(&mut self) {
        let (buzzer_result, led_result) = join!(self.buzzer.beep_nak(), self.led.turn_red_on_1s());

        buzzer_result.unwrap_or_else(|err| {
            error!("Failed to buzz: {err}");
        });

        led_result.unwrap_or_else(|err| {
            error!("Failed to set LED: {err}");
        });
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
                buzzer: GPIOBuzzer::new(PWM_CHANNEL_BUZZER)?,
                led: SpiLed::new()?,
            })
        }
    }
}
