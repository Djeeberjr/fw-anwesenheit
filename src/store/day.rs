use core::fmt::Write;

use embedded_sdmmc::ShortFileName;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Day(u32);

impl Day {
    const SECONDS_PER_DAY: u64 = 86_400;

    pub fn new(daystamp: u32) -> Self {
        Day(daystamp)
    }

    pub fn new_from_timestamp(time: u64) -> Self {
        let day = time / Self::SECONDS_PER_DAY;

        if day > u32::MAX as u64 {
            // TBH this would only happen if about 11 million years have passed
            // I sure hope i don't have to work on this project any more then
            // So we just cap it at this
            Day(u32::MAX)
        } else {
            Day(day as u32)
        }
    }

    pub fn to_timestamp(self) -> u64 {
        (self.0 as u64) * Self::SECONDS_PER_DAY
    }

    pub fn to_string(self) -> heapless::String<8> {
        let mut s: heapless::String<8> = heapless::String::new();
        write!(s, "{:08X}", self.0).unwrap();
        s
    }

    pub fn from_hex_str(s: &str) -> Result<Self, &'static str> {
        if s.len() > 8 {
            return Err("hex string too long");
        }

        u32::from_str_radix(s, 16)
            .map_err(|_| "invalid hex string")
            .map(Day)
    }
}

impl From<u64> for Day {
    fn from(value: u64) -> Self {
        Self::new_from_timestamp(value)
    }
}

impl TryFrom<ShortFileName> for Day {
    type Error = ();

    fn try_from(value: ShortFileName) -> Result<Self, Self::Error> {
        let name = core::str::from_utf8(value.base_name()).map_err(|_| ())?;
        Self::from_hex_str(name).map_err(|_| ())
    }
}
