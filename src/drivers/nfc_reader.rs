use embassy_time::{Duration, Timer};
use esp_hal::{Async, uart::Uart};
use log::{debug, info, warn};

use crate::TallyPublisher;

#[embassy_executor::task]
pub async fn rfid_reader_task(mut uart_device: Uart<'static, Async>, chan: TallyPublisher) {
    let mut uart_buffer = [0u8; 64];

    loop {
        debug!("Looking for NFC...");
        match uart_device.read_async(&mut uart_buffer).await {
            Ok(n) => {
                let mut hex_str = heapless::String::<64>::new();
                for byte in &uart_buffer[..n] {
                    core::fmt::Write::write_fmt(&mut hex_str, format_args!("{:02X} ", byte)).ok();
                }
                info!("Read {n} bytes from UART: {hex_str}");

                match extract_id(&uart_buffer) {
                    Some(read) => {
                        chan.publish(read.try_into().unwrap()).await;
                    }
                    None => {
                        warn!("Invalid read from the RFID reader");
                    }
                };
            }
            Err(e) => {
                log::error!("Error reading from UART: {e}");
            }
        }
        Timer::after(Duration::from_millis(200)).await;
    }
}

/// Scans the UART output and retuns the first propper read ID
/// This ensures that only valid ID are parsed
///
/// A valid read looks like this:
/// The first byte is always 0x02 (Start of text)
/// Followed by 12 Bytes of chars
/// Ended by 0x03 (End of text)
pub fn extract_id(buffer: &[u8]) -> Option<[u8; 12]> {
    const STX: u8 = 0x02; // Start of Text ASCII char
    const ETX: u8 = 0x03; // End of Text ASCII char
    const ID_LENGTH: usize = 12;
    const MINIMUM_SEQUENCE: usize = ID_LENGTH + 2; // STX + 12 bytes + ETX

    if buffer.len() < MINIMUM_SEQUENCE {
        return None;
    }

    for window_start in 0..=buffer.len() - MINIMUM_SEQUENCE {
        if buffer[window_start] == STX {
            let id_end = window_start + ID_LENGTH + 1;

            if buffer[id_end] == ETX {
                let mut id = [0u8; ID_LENGTH];
                id.copy_from_slice(&buffer[window_start + 1..id_end]);
                return Some(id);
            }
        }
    }

    None
}
