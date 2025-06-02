use std::{error::Error, time::Duration};

use crate::hotspot::{HotspotError};

#[cfg(feature = "mock_pi")]
use crate::mock::MockHotspot;

#[cfg(not(feature = "mock_pi"))]
use crate::hotspot::NMHotspot;

pub trait StatusLed {
    fn turn_off(&mut self) -> Result<(), Box<dyn Error>>;

    fn turn_on(&mut self, color: rgb::RGB8) -> Result<(), Box<dyn Error>>;
}

pub trait Buzzer {
    fn modulated_tone(
        &mut self,
        frequency_hz: f64,
        duration: Duration,
    ) -> impl Future<Output = Result<(), Box<dyn Error>>> + std::marker::Send;
}

pub trait Hotspot {
    fn enable_hotspot(
        &self,
    ) -> impl std::future::Future<Output = Result<(), HotspotError>> + std::marker::Send;

    fn disable_hotspot(
        &self,
    ) -> impl std::future::Future<Output = Result<(), HotspotError>> + std::marker::Send;
}

/// Create a struct to manage the hotspot
/// Respects the `mock_pi` flag.
pub fn create_hotspot() -> Result<impl Hotspot, HotspotError> {
    #[cfg(feature = "mock_pi")]
    {
        Ok(MockHotspot {})
    }

    #[cfg(not(feature = "mock_pi"))]
    {
        NMHotspot::new_from_env()
    }
}
