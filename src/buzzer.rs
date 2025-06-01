use rppal::pwm::{Channel, Error, Polarity, Pwm};
use std::{future::Future, time::Duration};
use tokio::time::sleep;

const DEFAULT_PWM_CHANNEL_BUZZER: Channel = Channel::Pwm0; //PWM0 = GPIO18/Physical pin 12

pub trait Buzzer {
    fn modulated_tone(
        &mut self,
        frequency_hz: f64,
        duration: Duration,
    ) -> impl Future<Output = Result<(), Error>> + std::marker::Send;
}

pub struct GPIOBuzzer {
    pwm: Pwm,
}

impl GPIOBuzzer {
    pub fn new_from_channel(channel: Channel) -> Result<Self, Error> {
        // Enable with dummy values; we'll set frequency/duty in the tone method
        let duty_cycle: f64 = 0.5;
        let pwm = Pwm::with_frequency(channel, 1000.0, duty_cycle, Polarity::Normal, true)?;
        pwm.disable()?;

        Ok(GPIOBuzzer { pwm })
    }

    pub fn new_default() -> Result<Self, Error> {
        Self::new_from_channel(DEFAULT_PWM_CHANNEL_BUZZER)
    }
}

impl Buzzer for GPIOBuzzer {
    async fn modulated_tone(&mut self, frequency_hz: f64, duration: Duration) -> Result<(), Error> {
        self.pwm.set_frequency(frequency_hz, 0.5)?; // 50% duty cycle (square wave)
        self.pwm.enable()?;
        sleep(duration).await;
        self.pwm.disable()?;
        Ok(())
    }
}
