use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use smart_leds::SmartLedsWrite;
use std::error::Error;
use ws2812_spi::Ws2812;

use crate::hardware::StatusLed;

const SPI_CLOCK_SPEED: u32 = 3_800_000;

pub struct SpiLed {
    controller: Ws2812<Spi>,
}

impl SpiLed {
    pub fn new() -> Result<Self, rppal::spi::Error> {
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, SPI_CLOCK_SPEED, Mode::Mode0)?;
        let controller = Ws2812::new(spi);
        Ok(SpiLed { controller })
    }
}

impl StatusLed for SpiLed {
    fn turn_off(&mut self) -> Result<(), Box<dyn Error>> {
        self.controller
            .write(vec![rgb::RGB8::new(0, 0, 0)].into_iter())?;
        Ok(())
    }

    fn turn_on(&mut self, color: rgb::RGB8) -> Result<(), Box<dyn Error>> {
        self.controller.write(vec![color].into_iter())?;
        Ok(())
    }
}
