#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex}, channel::Channel, pubsub::{
        PubSubChannel, Publisher,
        WaitResult::{Lagged, Message},
    }, signal::Signal
};
use embassy_time::{Duration, Timer};
use log::{debug, info};
use static_cell::make_static;

use crate::{store::TallyID, webserver::start_webserver};

mod drivers;
mod feedback;
mod init;
mod store;
mod webserver;

static FEEDBACK_STATE: Signal<CriticalSectionRawMutex, feedback::FeedbackState> = Signal::new();

type TallyChannel = PubSubChannel<NoopRawMutex, TallyID, 8, 2, 1>;
type TallyPublisher = Publisher<'static, NoopRawMutex, TallyID, 8, 2, 1>;

#[esp_hal_embassy::main]
async fn main(mut spawner: Spawner) {
    let (uart_device, stack, _i2c, buzzer_gpio) =
        init::hardware::hardware_init(&mut spawner).await;

    wait_for_stack_up(stack).await;

    info!("Starting up...");

    let chan: &'static mut TallyChannel = make_static!(PubSubChannel::new());

    //start_webserver(&mut spawner, stack);

    let publisher = chan.publisher().unwrap();

    let mut rtc = drivers::rtc::RTCClock::new(_i2c).await;

    /****************************** Spawning tasks ***********************************/
    debug!("spawing NFC reader task...");
    spawner.must_spawn(drivers::nfc_reader::rfid_reader_task(
        uart_device,
        publisher,
    ));

    debug!("spawing feedback task..");
    spawner.must_spawn(feedback::feedback_task(buzzer_gpio));
    /******************************************************************************/

    let mut sub = chan.subscriber().unwrap();

    debug!("everything spawned");
    FEEDBACK_STATE.signal(feedback::FeedbackState::Startup);

    loop {
        rtc.get_time().await;
        info!("Current RTC time: {}", rtc.get_time().await);
        Timer::after(Duration::from_millis(1000)).await;

        // let wait_result = sub.next_message().await;
        // match wait_result {
        //     Lagged(_) => debug!("Lagged"),
        //     Message(msg) => debug!("Got message: {msg:?}"),
        // }
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
