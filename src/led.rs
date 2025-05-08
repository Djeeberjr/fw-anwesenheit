use std::time::Duration;

use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use smart_leds::SmartLedsWrite;
use tokio::time::sleep;
use ws2812_spi::Ws2812;

use crate::color::NamedColor;

pub struct Led {
    controller: Ws2812<Spi>,
}

const STATUS_DURATION: Duration = Duration::from_secs(2); // 2s sleep for all status led signals

impl Led {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 3_800_000, Mode::Mode0)?;
        let controller = Ws2812::new(spi);
        Ok(Led { controller })
    }

    pub async fn turn_green_on_1s(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.controller.write(NamedColor::Green.into_iter())?;
        sleep(STATUS_DURATION).await;
        self.controller.write(NamedColor::Off.into_iter())?;
        Ok(())
    }

    pub fn turn_off(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.controller.write(NamedColor::Off.into_iter())?;
        Ok(())
    }
}
