use log::debug;

use crate::{buzzer::Buzzer, hotspot::Hotspot, led::StatusLed};

pub struct MockBuzzer {}

impl Buzzer for MockBuzzer {
    async fn beep_ack(&mut self) -> Result<(), rppal::pwm::Error> {
        debug!("Mockbuzzer: ACK");
        Ok(())
    }

    async fn beep_nak(&mut self) -> Result<(), rppal::pwm::Error> {
        debug!("Mockbuzzer: NAK");
        Ok(())
    }
}

pub struct MockLed {}

impl StatusLed for MockLed {
    async fn turn_green_on_1s(&mut self) -> Result<(), rppal::spi::Error> {
        debug!("Mockled: Turn LED green for 1 sec");
        Ok(())
    }

    async fn turn_red_on_1s(&mut self) -> Result<(), rppal::spi::Error> {
        debug!("Mockled: Turn LED red for 1 sec");
        Ok(())
    }
}

pub struct MockHotspot {}

impl Hotspot for MockHotspot {
    async fn enable_hotspot(&self) -> Result<(), crate::hotspot::HotspotError> {
        debug!("Mockhotspot: Enable hotspot");
        Ok(())
    }

    async fn disable_hotspot(&self) -> Result<(), crate::hotspot::HotspotError> {
        debug!("Mockhotspot: Disable hotspot");
        Ok(())
    }
}
