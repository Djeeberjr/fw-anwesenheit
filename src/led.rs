use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use smart_leds::{RGB8, SmartLedsWrite};
use ws2812_spi::Ws2812;

pub struct Led {
    controller: Ws2812<Spi>,
}

impl Led {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 3_000_000, Mode::Mode0)?;
        let controller = Ws2812::new(spi);
        Ok(Led { controller })
    }

    pub fn turn_green_on(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let green = [RGB8 { r: 0, g: 255, b: 0 }];
        self.controller.write(green.iter().cloned())?;
        Ok(())
    }

    pub fn turn_off(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let off = [RGB8 { r: 0, g: 0, b: 0 }];
        self.controller.write(off.iter().cloned())?;
        Ok(())
    }
}
