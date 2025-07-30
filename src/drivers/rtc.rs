use ds3231::{Alarm1Config, DS3231Error, Seconds, DS3231};
use embassy_time::{Timer, Duration};
use esp_hal::{i2c::{self, master::I2c}, peripherals, Async};
use log::{debug, error, info};

use crate::{drivers, init, UTC_TIME};

const RTC_ADDRESS: u8 =  0x57;

#[embassy_executor::task]
async fn rtc_task(
    i2c: i2c::master::I2c<'static, Async>,
    sqw_pin: peripherals::GPIO21<'static>,
) {
    debug!("init rtc interrupt");
    let mut rtc_interrupt = init::hardware::setup_rtc_iterrupt(sqw_pin).await;
    debug!("configuring rtc");
    let mut rtc = drivers::rtc::rtc_config(i2c).await;

    let mut utc_time = UTC_TIME.lock().await;
    let timestamp_result = drivers::rtc::read_rtc_time(&mut rtc).await;
    *utc_time = timestamp_result.unwrap_or(0);

    loop {
        debug!("Waiting for RTC interrupt...");
        rtc_interrupt.wait_for_falling_edge().await;
        debug!("RTC interrupt triggered");
        utc_time = UTC_TIME.lock().await;
        let timestamp_result = drivers::rtc::read_rtc_time(&mut rtc).await;
        *utc_time = timestamp_result.unwrap_or(0);
        Timer::after(Duration::from_secs(1)).await; // Debounce delay
    }
}

pub async fn rtc_config(i2c: I2c<'static, Async>) -> DS3231<I2c<'static, Async>> {
    let mut rtc: DS3231<I2c<'static, Async>> = DS3231::new(i2c, RTC_ADDRESS);
    let daily_alarm = Alarm1Config::AtTime {
        hours: 0,   // set alarm every day 00:00:00 to sync time
        minutes: 0,
        seconds: 10,
        is_pm: None, // 24-hour mode
    };
    // Replace 'main::UTC_TIME' with the correct path to UTC_TIME, for example 'crate::UTC_TIME'
    let mut utc_time;
    {
        utc_time = crate::UTC_TIME.lock().await;
    }

    let naive_dt = chrono::NaiveDateTime::from_timestamp_opt(*utc_time as i64, 0)
        .expect("Invalid timestamp for NaiveDateTime");
    rtc.set_datetime(&naive_dt).await.unwrap_or_else(|e| {
        error!("Failed to set RTC datetime: {:?}", e);
        panic!();
    });
    
    if let Err(e) = rtc.set_alarm1(&daily_alarm).await {
        error!("Failed to configure RTC: {:?}", e);
        panic!();
    }
    rtc
}

pub async fn read_rtc_time<'a>(rtc: &'a mut DS3231<I2c<'static, Async>>) -> Result<u64, DS3231Error<esp_hal::i2c::master::Error>> {
    match rtc.datetime().await {
        Ok(datetime) => {
            let utc_time = datetime.and_utc().timestamp() as u64;
            Ok(utc_time)
        }
        Err(e) => {
            error!("Failed to read RTC datetime: {:?}", e);
            Err(e)
        }
    }
}


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