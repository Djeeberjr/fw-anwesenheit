use rppal::pwm::{Pwm, Channel, Polarity};
use tokio::time::sleep;
use std::{error::Error, time::Duration};

pub struct GPIOBuzzer {
    pwm: Pwm,
}

impl GPIOBuzzer {
    /// Create a new GPIOBuzzer instance.
    /// 0.5 duty cyle 
    /// # Arguments
    /// * "channel" - PWM channel for buzzer PWM0 = GPIO 12 / PWM1 = GPIO 13
    pub fn new(channel: Channel) -> Result<Self, Box<dyn Error>> {
        // Enable with dummy values; we'll set frequency/duty in the tone method
        let duty_cycle:f64 = 0.5;
        let pwm = Pwm::with_frequency(channel, 1000.0, duty_cycle, Polarity::Normal, true)?;
        pwm.disable()?; // Start disabled

        Ok(GPIOBuzzer { pwm })
    }

        /// Play a tone using hardware PWM on supported GPIO pins.
    ///
    /// # Arguments
    /// * `frequency` - Frequency in Hz.
    /// * `duration_ms` - Duration in milliseconds.
    async fn modulated_tone(&mut self, frequency: f64, duration_ms: u64) -> Result<(), Box<dyn Error>> {
        self.pwm.set_frequency(frequency, 0.5)?; // 50% duty cycle (square wave)
        self.pwm.enable()?;
        sleep(Duration::from_millis(duration_ms)).await;
        self.pwm.disable()?;
        Ok(())
    }

    pub async fn beep_ack(&mut self) -> Result<(), Box<dyn Error>>{
        let sleep_ms: u64 = 100;
        self.modulated_tone(750.0, 100).await?;
        sleep(Duration::from_millis(sleep_ms)).await;
        self.modulated_tone(1200.0,100).await?;
        sleep(Duration::from_millis(sleep_ms)).await;
        self.modulated_tone(2300.0,100).await?;
        Ok(())
    }

    pub async fn beep_nak(&mut self) -> Result<(), Box<dyn Error>>{
        self.modulated_tone(2300.0,100).await?;
        self.modulated_tone(2300.0,100).await?;
        Ok(())
    }

    pub async fn beep_unnkown(&mut self) -> Result<(), Box<dyn Error>>{
        let sleep_ms: u64 = 100;
        self.modulated_tone(2300.0,100).await?;
        sleep(Duration::from_millis(sleep_ms)).await;
        self.modulated_tone(2300.0,100).await?;
        sleep(Duration::from_millis(sleep_ms)).await;
        self.modulated_tone(2300.0,100).await?;
        Ok(())
    }
}
