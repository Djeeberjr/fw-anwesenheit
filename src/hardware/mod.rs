use anyhow::Result;
use std::time::Duration;

mod gpio_buzzer;
mod hotspot;
mod mock;
mod spi_led;

pub use gpio_buzzer::GPIOBuzzer;
pub use mock::{MockBuzzer, MockHotspot, MockLed};
pub use spi_led::SpiLed;

pub trait StatusLed {
    fn turn_off(&mut self) -> Result<()>;

    fn turn_on(&mut self, color: rgb::RGB8) -> Result<()>;
}

pub trait Buzzer {
    fn modulated_tone(
        &mut self,
        frequency_hz: f64,
        duration: Duration,
    ) -> impl Future<Output = Result<()>> + std::marker::Send;
}

