use embassy_time::{Duration, Timer};
use esp_hal::{uart::Uart, Async};
use log::{ debug, info };

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
                chan.publish([1, 0, 2, 5, 0, 8, 12, 15]).await;
            }
            Err(e) => {
                log::error!("Error reading from UART: {e}");
            }
        }
        Timer::after(Duration::from_millis(200)).await;
    }
}