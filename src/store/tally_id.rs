use core::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TallyID([u8; 6]);

impl FromStr for TallyID {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.as_bytes().try_into()
    }
}

impl TryFrom<heapless::String<12>> for TallyID {
    type Error = ();

    fn try_from(value: heapless::String<12>) -> Result<Self, Self::Error> {
        let bytes = value.as_bytes();

        let mut out: [u8; 6] = [0; 6];
        for i in 0..6 {
            let hi = hex_val(bytes[2 * i])?;
            let lo = hex_val(bytes[2 * i + 1])?;
            out[i] = (hi << 4) | lo;
        }

        Ok(TallyID(out))
    }
}

fn hex_val(b: u8) -> Result<u8, ()> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(()),
    }
}

impl From<TallyID> for heapless::String<12> {
    fn from(value: TallyID) -> Self {
        const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";
        let mut s: Self = Self::new();

        for &b in &value.0 {
            // Should be safe to unwrap since the string is already long enough
            s.push(HEX_CHARS[(b >> 4) as usize] as char).unwrap();
            s.push(HEX_CHARS[(b & 0x0F) as usize] as char).unwrap();
        }
        s
    }
}

/// From a array of hex chars
impl TryFrom<&[u8]> for TallyID {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 12 {
            return Err(());
        }

        let mut out: [u8; 6] = [0; 6];
        for i in 0..6 {
            let hi = hex_val(value[2 * i])?;
            let lo = hex_val(value[2 * i + 1])?;
            out[i] = (hi << 4) | lo;
        }

        Ok(TallyID(out))
    }
}

impl Serialize for TallyID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s: heapless::String<12> = (*self).into();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for TallyID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        TallyID::from_str(s).map_err(|_| de::Error::custom("Failed to parse Tally ID"))
    }
}
