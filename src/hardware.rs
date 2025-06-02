use anyhow::Result;
use std::time::Duration;

#[cfg(feature = "mock_pi")]
use crate::mock::MockHotspot;

#[cfg(not(feature = "mock_pi"))]
use crate::hotspot::NMHotspot;

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

pub trait Hotspot {
    fn enable_hotspot(&self) -> impl std::future::Future<Output = Result<()>> + std::marker::Send;

    fn disable_hotspot(&self) -> impl std::future::Future<Output = Result<()>> + std::marker::Send;
}

/// Create a struct to manage the hotspot
/// Respects the `mock_pi` flag.
pub fn create_hotspot() -> Result<impl Hotspot> {
    #[cfg(feature = "mock_pi")]
    {
        Ok(MockHotspot {})
    }

    #[cfg(not(feature = "mock_pi"))]
    {
        NMHotspot::new_from_env()
    }
}
