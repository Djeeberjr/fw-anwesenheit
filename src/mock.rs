use std::{error::Error, time::Duration};

use log::debug;
use tokio::time::sleep;

use crate::hardware::{Buzzer, Hotspot, StatusLed};

pub struct MockBuzzer {}

impl Buzzer for MockBuzzer {
    async fn modulated_tone(
        &mut self,
        frequency_hz: f64,
        duration: Duration,
    ) -> Result<(), Box<dyn Error>> {
        debug!("MockBuzzer: modulte tone: {frequency_hz} Hz");
        sleep(duration).await;
        Ok(())
    }
}

pub struct MockLed {}

impl StatusLed for MockLed {
    fn turn_off(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Turn mock LED off");
        Ok(())
    }

    fn turn_on(&mut self, color: rgb::RGB8) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Turn mock LED on to: {color}");
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
