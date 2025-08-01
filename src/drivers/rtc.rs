use embassy_time::{Duration, Timer};
use log::info;

#[embassy_executor::task]
pub async fn rtc_task() {
    info!("RTC task started");
    // Initialize I2C and RTC here

    loop {
        // Read RTC time and update UTC_TIME signal
        // let utc_time = read_rtc_time(&mut rtc).await.unwrap();
        // UTC_TIME.signal(utc_time);

        // Simulate waiting for an interrupt or event
        Timer::after(Duration::from_millis(1000)).await;
        info!("RTC tick");
    }
}








/* ************************************************************************************** */


// use ds3231::{Alarm1Config, DS3231, DS3231Error, Seconds};
// use embassy_time::{Duration, Timer};
// use esp_hal::{
//     Async,
//     i2c::{self, master::I2c},
//     peripherals,
// };
// use log::{debug, error, info};

// use crate::{UTC_TIME, drivers, init};

// const RTC_ADDRESS: u8 = 0x57;

// #[embassy_executor::task]
// pub async fn rtc_task(
//     //i2c: i2c::master::I2c<'static, Async>,
//     //sqw_pin: peripherals::GPIO21<'static>,
// ) {
//     //UTC_TIME.signal(155510);

//     // debug!("init rtc interrupt");
//     // let mut rtc_interrupt = init::hardware::setup_rtc_iterrupt(sqw_pin).await;
//     // debug!("configuring rtc");
//     // let mut rtc = drivers::rtc::rtc_config(i2c).await;

//     // let timestamp_result = drivers::rtc::read_rtc_time(&mut rtc).await;
//     // UTC_TIME.signal(timestamp_result.unwrap());

//     debug!("rtc up");
//     loop {
//         info!("Current UTC time: {}", UTC_TIME.wait().await);
//         // debug!("Waiting for RTC interrupt...");
//         // rtc_interrupt.wait_for_falling_edge().await;
//         // debug!("RTC interrupt triggered");
//         // let timestamp_result = drivers::rtc::read_rtc_time(&mut rtc).await;
//         // UTC_TIME.signal(timestamp_result.unwrap());
//         // Timer::after(Duration::from_secs(1)).await; // Debounce delay
//     }
// }

// pub async fn rtc_config(i2c: I2c<'static, Async>) -> DS3231<I2c<'static, Async>> {
//     let mut rtc: DS3231<I2c<'static, Async>> = DS3231::new(i2c, RTC_ADDRESS);
//     let daily_alarm = Alarm1Config::AtTime {
//         hours: 0, // set alarm every day 00:00:00 to sync time
//         minutes: 0,
//         seconds: 10,
//         is_pm: None, // 24-hour mode
//     };

    // let naive_dt = chrono::NaiveDateTime::from_timestamp_opt(*utc_time as i64, 0)
    //     .expect("Invalid timestamp for NaiveDateTime");
    // rtc.set_datetime(&naive_dt).await.unwrap_or_else(|e| {
    //     error!("Failed to set RTC datetime: {:?}", e);
    //     panic!();
    // });

    // if let Err(e) = rtc.set_alarm1(&daily_alarm).await {
    //     error!("Failed to configure RTC: {:?}", e);
    //     panic!();
    // }
//     rtc
// }

// pub async fn read_rtc_time<'a>(
//     rtc: &'a mut DS3231<I2c<'static, Async>>,
// ) -> Result<u64, DS3231Error<esp_hal::i2c::master::Error>> {
//     match rtc.datetime().await {
//         Ok(datetime) => {
//             let utc_time = datetime.and_utc().timestamp() as u64;
//             Ok(utc_time)
//         }
//         Err(e) => {
//             error!("Failed to read RTC datetime: {:?}", e);
//             Err(e)
//         }
//     }
// }

// TODO Update time when device is connected other device over Wifi
/* pub async fn update_rtc_time<'a>(rtc: &'a mut DS3231<I2c<'static, Async>>, datetime: u64) -> Result<(), DS3231Error<esp_hal::i2c::master::Error>> {

    match rtc.set_datetime(datetime).await {
        info!("RTC datetime updated to: {}", datetime);
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failed to update RTC datetime: {:?}", e);
            Err(e)
        }
    }
}
 */
