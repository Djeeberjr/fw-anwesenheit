mod id_mapping;
pub mod persistence;
mod id_store;

use heapless::String;
pub use id_mapping::{IDMapping, Name};
pub use id_store::{IDStore,AttendanceDay};

pub type TallyID = [u8; 6];
pub type Date = [u8; 10];

pub fn hex_string_to_tally_id(s: &str) -> Option<TallyID> {
    let bytes = s.as_bytes();
    if bytes.len() != 12 {
        return None;
    }

    let mut out: TallyID = [0;6];
    for i in 0..6 {
        let hi = hex_val(bytes[2 * i])?;
        let lo = hex_val(bytes[2 * i + 1])?;
        out[i] = (hi << 4) | lo;
    }
    Some(out)
}

pub fn tally_id_to_hex_string(bytes: TallyID) -> Option<String<12>> {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut s: String<12> = String::new();

    for &b in &bytes {
        s.push(HEX_CHARS[(b >> 4) as usize] as char).ok()?;
        s.push(HEX_CHARS[(b & 0x0F) as usize] as char).ok()?;
    }

    Some(s)
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

