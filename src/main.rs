#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]
#![warn(clippy::unwrap_used)]

use alloc::rc::Rc;
use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    mutex::Mutex,
    pubsub::{
        PubSubChannel, Publisher, Subscriber,
        WaitResult::{Lagged, Message},
    },
    signal::Signal,
};
use embassy_time::{Duration, Timer};
use esp_hal::gpio::InputConfig;
use esp_hal::gpio::{AnyPin, Input};
use log::{debug, info};
use static_cell::StaticCell;

extern crate alloc;

use crate::{
    init::{hardware::AppHardware, sd_card::SDCardPersistence},
    store::{IDStore, day::Day, mapping_loader::MappingLoader, tally_id::TallyID},
    webserver::start_webserver,
};

mod drivers;
mod feedback;
mod init;
mod store;
mod webserver;

static FEEDBACK_STATE: Signal<CriticalSectionRawMutex, feedback::FeedbackState> = Signal::new();

type TallyChannel = PubSubChannel<NoopRawMutex, TallyID, 8, 2, 1>;
type TallyPublisher = Publisher<'static, NoopRawMutex, TallyID, 8, 2, 1>;
type TallySubscriber = Subscriber<'static, NoopRawMutex, TallyID, 8, 2, 1>;
type UsedStore = IDStore<SDCardPersistence>;

static CHAN: StaticCell<TallyChannel> = StaticCell::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let app_hardware = AppHardware::init(spawner).await.unwrap();

    info!("Starting up...");

    let mut rtc = drivers::rtc::RTCClock::new(app_hardware.i2c).await;

    let current_day: Day = rtc.get_time().await.into();

    let shared_sdcard: Rc<Mutex<CriticalSectionRawMutex, SDCardPersistence>> =
        Rc::new(Mutex::new(app_hardware.sdcard));

    let store: UsedStore = IDStore::new_from_storage(shared_sdcard.clone(), current_day).await;
    let shared_store = Rc::new(Mutex::new(store));

    let mapping_loader = MappingLoader::new(shared_sdcard.clone());

    let chan: &'static mut TallyChannel = CHAN.init(PubSubChannel::new());
    let publisher: TallyPublisher = chan.publisher().unwrap();
    let mut sub: TallySubscriber = chan.subscriber().unwrap();

    wait_for_stack_up(app_hardware.network_stack).await;

    start_webserver(
        spawner,
        app_hardware.network_stack,
        shared_store.clone(),
        chan,
        mapping_loader,
    );

    /****************************** Spawning tasks ***********************************/
    debug!("spawing NFC reader task...");
    spawner.must_spawn(drivers::nfc_reader::rfid_reader_task(
        app_hardware.uart,
        publisher,
    ));

    debug!("spawing feedback task..");
    spawner.must_spawn(feedback::feedback_task(
        app_hardware.led,
        app_hardware.buzzer,
    ));

    debug!("spawn sd detect task");
    spawner.must_spawn(sd_detect_task(app_hardware.sd_present));
    /******************************************************************************/

    debug!("everything spawned");
    FEEDBACK_STATE.signal(feedback::FeedbackState::Startup);

    loop {
        let wait_result = sub.next_message().await;
        match wait_result {
            Lagged(_) => debug!("Lagged"),
            Message(msg) => {
                debug!("Got message: {msg:?}");

                let day: Day = rtc.get_time().await.into();
                let added = shared_store.lock().await.add_id(msg, day).await;

                if added {
                    FEEDBACK_STATE.signal(feedback::FeedbackState::Ack);
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn sd_detect_task(sd_det_gpio: AnyPin<'static>) {
    let mut sd_det = Input::new(sd_det_gpio, InputConfig::default());
    sd_det.wait_for(esp_hal::gpio::Event::AnyEdge).await;

    loop {
        sd_det.wait_for_any_edge().await;
        {
            if sd_det.is_high() {
                FEEDBACK_STATE.signal(feedback::FeedbackState::Ack);
                debug!("card insert");
            }
            //card is not insert on low
            else {
                FEEDBACK_STATE.signal(feedback::FeedbackState::Nack);
                debug!("card removed");
            }
        }
        //debounce time
        Timer::after(Duration::from_millis(100)).await;
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
