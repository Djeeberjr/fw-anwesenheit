use embassy_time::{Duration, Timer};
use esp_hal::{Async, uart::Uart};
use log::{debug, info};

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

                // The first byte is always 0x02 (Start of text) 
                // Followed by 12 Bytes of chars
                // Ended by 0x03 (End of text)
                chan.publish(uart_buffer[1..13].try_into().unwrap()).await;
            }
            Err(e) => {
                log::error!("Error reading from UART: {e}");
            }
        }
        Timer::after(Duration::from_millis(200)).await;
    }
}
