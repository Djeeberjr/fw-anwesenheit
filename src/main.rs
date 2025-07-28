#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex}, mutex::Mutex, pubsub::{
        PubSubChannel, Publisher,
        WaitResult::{Lagged, Message},
    }
};
use embassy_time::{Duration, Timer};
use esp_alloc::psram_allocator;
use esp_hal::{i2c, peripherals, Async};
use esp_hal::uart::Uart;
use log::{debug, info};
use static_cell::make_static;

use crate::{store::TallyID, webserver::start_webserver};

mod init;
mod drivers;
mod store;
mod webserver;

type TallyChannel = PubSubChannel<NoopRawMutex, TallyID, 8, 2, 1>;
type TallyPublisher = Publisher<'static, NoopRawMutex, TallyID, 8, 2, 1>;

static UTC_TIME: Mutex<CriticalSectionRawMutex, u64> = Mutex::new(0);

#[esp_hal_embassy::main]
async fn main(mut spawner: Spawner) {
    let (uart_device, stack, i2c, sqw_pin) = init::hardware::hardware_init(&mut spawner).await;

    wait_for_stack_up(stack).await;

    let chan: &'static mut TallyChannel = make_static!(PubSubChannel::new());

    start_webserver(&mut spawner, stack);

    let publisher = chan.publisher().unwrap();

    spawner.must_spawn(drivers::nfc_reader::rfid_reader_task(uart_device, publisher));
    spawner.must_spawn(rtc_task(i2c, sqw_pin));

    let mut sub = chan.subscriber().unwrap();
    loop {
        let wait_result = sub.next_message().await;
        match wait_result {
            Lagged(_) => debug!("Lagged"),
            Message(msg) => debug!("Got message: {msg:?}"),
        }
    }
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
async fn rtc_task(
    mut i2c: i2c::master::I2c<'static, Async>,
    sqw_pin: peripherals::GPIO21<'static>,
) {
    let mut rtc_interrupt = init::hardware::rtc_init_iterrupt(sqw_pin).await;
    let mut rtc = init::hardware::rtc_config(i2c).await;

    loop {
        rtc_interrupt.wait_for_falling_edge().await;
        debug!("RTC interrupt triggered");
        if let Ok(datetime) = rtc.datetime().await {
            let mut utc_time = UTC_TIME.lock().await;
            *utc_time = datetime.and_utc().timestamp() as u64;
            info!("RTC updated UTC_TIME: {}", *utc_time);
        } else {
            info!("Failed to read RTC datetime");
        }
    }
}