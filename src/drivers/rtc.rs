use core::time;

use chrono::NaiveDate;
use ds3231::{
    Alarm1Config, Config, DS3231, DS3231Error, InterruptControl, Oscillator, Seconds,
    SquareWaveFrequency, TimeRepresentation,
};
use embassy_time::{Duration, Timer, WithTimeout};
use esp_hal::{
    Async,
    i2c::{self, master::I2c},
    peripherals,
};
use log::{debug, error, info};

use crate::{FEEDBACK_STATE, UTC_TIME, drivers, feedback, init};
use chrono::{NaiveDateTime, TimeZone, Utc};

include!(concat!(env!("OUT_DIR"), "/build_time.rs"));

const RTC_ADDRESS: u8 = 0x68;

#[embassy_executor::task]
pub async fn rtc_task(
    i2c: i2c::master::I2c<'static, Async>,
    sqw_pin: peripherals::GPIO21<'static>,
) {
    UTC_TIME.signal(BUILD_UNIX_TIME);
    info!("Build time: {}", BUILD_UNIX_TIME);

    // i2c.write_async(RTC_ADDRESS, &[0x0E, 0b00000000]) // Clear control register
    // .await
    // .unwrap_or_else(|e| {
    //     FEEDBACK_STATE.signal(feedback::FeedbackState::Error);
    //     error!("Failed to clear RTC control register: {:?}", e);
    // });

    // debug!("init rtc interrupt");
    // let mut rtc_interrupt = init::hardware::setup_rtc_iterrupt(sqw_pin).await;

    debug!("configuring rtc");
    let mut rtc = drivers::rtc::rtc_config(i2c).await;

    debug!("rtc up");
    loop {
        //set_rtc_alarm(&mut rtc).await;
        // debug!("Waiting for RTC interrupt...");
        // rtc_interrupt.wait_for_falling_edge().await;
        // debug!("RTC interrupt triggered");

        Timer::after(Duration::from_millis(1000)).await;
        //TODO use pub sub channel or something similar to send the time when needed
        let timestamp = drivers::rtc::read_rtc_time(&mut rtc).await;
        match timestamp {
            Ok(ts) => {
                UTC_TIME.signal(ts);
                info!("Current UTC time: {}", UTC_TIME.wait().await);
            }
            Err(e) => {
                FEEDBACK_STATE.signal(feedback::FeedbackState::Error);
                error!("Failed to read RTC datetime: {:?}", e);
            }
        }
    }
}

pub async fn rtc_config(i2c: I2c<'static, Async>) -> DS3231<I2c<'static, Async>> {
    let mut rtc: DS3231<I2c<'static, Async>> = DS3231::new(i2c, RTC_ADDRESS);
    let naive_dt = Utc
        .timestamp_opt(BUILD_UNIX_TIME as i64, 0)
        .single()
        .unwrap()
        .naive_utc();

    let rtc_config = Config {
        time_representation: TimeRepresentation::TwentyFourHour,
        square_wave_frequency: SquareWaveFrequency::Hz1,
        interrupt_control: InterruptControl::Interrupt, // Enable interrupt mode
        battery_backed_square_wave: false,
        oscillator_enable: Oscillator::Disabled,
    };

    match rtc.configure(&rtc_config).await {
        Ok(_) => info!("DS3231 configured successfully"),
        Err(e) => {
            info!("Failed to configure DS3231: {:?}", e);
            panic!("DS3231 configuration failed");
        }
    }

    rtc.set_datetime(&naive_dt).await.unwrap_or_else(|e| {
        FEEDBACK_STATE.signal(feedback::FeedbackState::Error);
        error!("Failed to set RTC datetime: {:?}", e);
    });
    info!("RTC datetime set to: {}", naive_dt);


    match rtc.status().await {
        Ok(mut status) => {
            status.set_alarm1_flag(false);
            status.set_alarm2_flag(false);
            match rtc.set_status(status).await {
                Ok(_) => info!("Alarm flags cleared"),
                Err(e) => info!("Failed to clear alarm flags: {:?}", e),
            }
        }
        Err(e) => info!("Failed to read status: {:?}", e),
    }

    rtc
}


pub async fn read_rtc_time<'a>(
    rtc: &'a mut DS3231<I2c<'static, Async>>,
) -> Result<u64, DS3231Error<esp_hal::i2c::master::Error>> {
    let timestamp_result = rtc.datetime().await?;
    Ok(timestamp_result.and_utc().timestamp() as u64)
}

//     match rtc.datetime().await {
//         Ok(datetime) => {
//             let utc_time = datetime.and_utc().timestamp() as u64;
//             Ok(utc_time)
//         }
//         Err(e) => {
//             FEEDBACK_STATE.signal(feedback::FeedbackState::Error);
//             error!("Failed to read RTC datetime: {:?}", e);
//             Err(e)
//         }
//     }
// }

    // let alarm_config = Alarm1Config::AtSeconds { seconds: 0};

    // match rtc.set_alarm1(&alarm_config).await {
    //     Ok(_) => info!("Alarm 1 set to trigger at seconds"),
    //     Err(e) => {
    //         FEEDBACK_STATE.signal(feedback::FeedbackState::Error);
    //         error!("Failed to set Alarm 1: {:?}", e);
    //     }
    // }

/* ************************************************************************************** */

// #[embassy_executor::task]
// pub async fn rtc_task() {
//     info!("RTC task started");
//     // Initialize I2C and RTC here

//     loop {
//         // Read RTC time and update UTC_TIME signal
//         // let utc_time = read_rtc_time(&mut rtc).await.unwrap();
//         // UTC_TIME.signal(utc_time);

//         // Simulate waiting for an interrupt or event
//         Timer::after(Duration::from_millis(1000)).await;
//         info!("RTC tick");
//     }
// }

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
