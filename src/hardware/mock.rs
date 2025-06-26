use anyhow::Result;
use log::debug;
use std::time::Duration;
use tokio::time::sleep;

use crate::hardware::{Buzzer, Hotspot, StatusLed};

pub struct MockBuzzer {}

impl Buzzer for MockBuzzer {
    async fn modulated_tone(&mut self, frequency_hz: f64, duration: Duration) -> Result<()> {
        debug!("MockBuzzer: modulte tone: {frequency_hz} Hz");
        sleep(duration).await;
        Ok(())
    }
}

pub struct MockLed {}

impl StatusLed for MockLed {
    fn turn_off(&mut self) -> Result<()> {
        debug!("Turn mock LED off");
        Ok(())
    }

    fn turn_on(&mut self, color: rgb::RGB8) -> Result<()> {
        debug!("Turn mock LED on to: {color}");
        Ok(())
    }
}

pub struct MockHotspot {}

impl Hotspot for MockHotspot {
    async fn enable_hotspot(&self) -> Result<()> {
        debug!("Mockhotspot: Enable hotspot");
        Ok(())
    }

    async fn disable_hotspot(&self) -> Result<()> {
        debug!("Mockhotspot: Disable hotspot");
        Ok(())
    }
}
