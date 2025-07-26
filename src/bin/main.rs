#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_time::{Duration, Timer};
use esp_hal::Async;
use esp_hal::uart::Uart;
use log::{debug, info};

use crate::webserver::start_webserver;

mod init;
mod webserver;
mod store;

#[esp_hal_embassy::main]
async fn main(mut spawner: Spawner) {
    let (uart_device, stack) = init::hardware_init(&mut spawner).await;

    wait_for_stack_up(stack).await;

    start_webserver(&mut spawner, stack);
    spawner.must_spawn(rfid_reader_task(uart_device));
}

async fn wait_for_stack_up(stack: Stack<'static>) {
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
        if stack.is_config_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn rfid_reader_task(mut uart_device: Uart<'static, Async>) {
    let mut uart_buffer = [0u8; 64];

    loop {
        debug!("Looking for NFC...");
        match uart_device.read_async(&mut uart_buffer).await {
            Ok(n) => {
                let mut hex_str = heapless::String::<128>::new();
                for byte in &uart_buffer[..n] {
                    core::fmt::Write::write_fmt(&mut hex_str, format_args!("{:02X} ", byte)).ok();
                }
                info!("Read {n} bytes from UART: {hex_str}");
            }
            Err(e) => {
                log::error!("Error reading from UART: {e}");
            }
        }
        Timer::after(Duration::from_millis(200)).await;
    }
}
