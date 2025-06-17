use anyhow::Result;
use rgb::RGB8;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use smart_leds::SmartLedsWrite;
use ws2812_spi::Ws2812;

use crate::{feedback, hardware::StatusLed};

const SPI_CLOCK_SPEED: u32 = 3_800_000;

pub enum CurrentStatus {
    Ready,
    Hotspot,
}

impl CurrentStatus {
    pub fn color(&self) -> RGB8 {
        match self {
            CurrentStatus::Ready => RGB8::new(0, 50, 0),
            CurrentStatus::Hotspot => RGB8::new(0, 0, 50),
        }
    }
}

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
    fn turn_off(&mut self) -> Result<()> {
        self.controller
            .write(vec![feedback::CURRENTSTATUS.color()].into_iter())?;
        Ok(())
    }

    fn turn_on(&mut self, color: rgb::RGB8) -> Result<()> {
        self.controller.write(vec![color].into_iter())?;
        Ok(())
    }
}
