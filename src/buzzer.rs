use rppal::gpio::{Gpio, OutputPin};
use std::{error::Error, time};
use tokio::time::sleep;

pub struct GPIOBuzzer {
    pin: OutputPin,
}

impl GPIOBuzzer {
    pub fn new(pin_num: u8) -> Result<Self, Box<dyn Error>> {
        let gpio = Gpio::new()?;
        let pin = gpio.get(pin_num)?.into_output();

        Ok(GPIOBuzzer { pin })
    }

    /// Emits a sound on a passive buzzer.
    async fn modulated_tone(&mut self, carrier_hz: u32, sound_hz: u32, duration_ms: u64) {
        let carrier_period =
            time::Duration::from_micros((1_000_000.0 / carrier_hz as f64 / 2.0) as u64);
        let mod_period = 1_000.0 / sound_hz as f64; // in ms
        let total_cycles = duration_ms as f64 / mod_period;

        for _ in 0..total_cycles as u64 {
            // Modulation on: Carrier on for mod_period / 2
            let cycles_on = (carrier_hz as f64 * (mod_period / 2.0) / 1000.0) as u64;
            for _ in 0..cycles_on {
                self.pin.set_high();
                sleep(carrier_period).await;
                self.pin.set_low();
                sleep(carrier_period).await;
            }

            // Modulation off: Carrier on for mod_period / 2
            let pause = time::Duration::from_millis((mod_period / 2.0) as u64);
            sleep(pause).await;
        }
    }
    pub async fn beep_ack(&mut self) {
        // carrier  = 2300 Hz, sound = 440 Hz, duration = 1 sec
        self.modulated_tone(2300, 659, 400).await;
        self.modulated_tone(2300, 784,100).await;
    }

    pub async fn beep_nak(&mut self) {
        // carrier  = 2300 Hz, sound = 440 Hz, duration = 1 sec
        self.modulated_tone(2300, 659, 400).await;
        self.modulated_tone(2300, 523, 100).await;
    }

    pub async fn beep_unnkown(&mut self) {
        self.modulated_tone(2300, 784, 150).await;
        self.modulated_tone(2300, 659, 150).await;
        self.modulated_tone(2300, 500, 150).await;
    }
}
