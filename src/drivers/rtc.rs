use chrono::{TimeZone, Utc};
use ds3231::{
    Config, DS3231, DS3231Error, InterruptControl, Oscillator, SquareWaveFrequency,
    TimeRepresentation,
};
use esp_hal::{
    Async,
    i2c::{self, master::I2c},
};
use log::{debug, error, info};

use crate::{FEEDBACK_STATE, drivers, feedback};

include!(concat!(env!("OUT_DIR"), "/build_time.rs"));

const RTC_ADDRESS: u8 = 0x68;

const SECS_PER_DAY: u64 = 86_400;
const UNIX_OFFSET_DAYS: u64 = 719_163; // Days from 0000-03-01 to 1970-01-01
const UTC_PLUS_ONE: u64 = 3600;

pub struct RTCClock {
    dev: DS3231<I2c<'static, Async>>,
}

impl RTCClock {
    pub async fn new(i2c: i2c::master::I2c<'static, Async>) -> Self {
        debug!("configuring rtc...");
        let rtc = drivers::rtc::rtc_config(i2c).await;
        debug!("rtc up");

        RTCClock { dev: rtc }
    }

    pub async fn get_time(&mut self) -> u64 {
        match self.dev.datetime().await {
            Ok(datetime) => {
                let utc_time = datetime.and_utc().timestamp() as u64;
                utc_time
            }
            Err(e) => {
                FEEDBACK_STATE.signal(feedback::FeedbackState::Error);
                error!("Failed to read RTC datetime: {:?}", e);
                0
            }
        }
    }
}

fn unix_to_ymd_string(timestamp: u64) -> (u16, u8, u8) {
    // Apply UTC+1 offset
    let ts = timestamp + UTC_PLUS_ONE;

    // Convert to total days since UNIX epoch
    let days_since_epoch = ts / SECS_PER_DAY;

    // Convert to proleptic Gregorian date
    civil_from_days(days_since_epoch as i64 + UNIX_OFFSET_DAYS as i64)
}

// This function returns (year, month, day).
// Based on the algorithm by Howard Hinnant.
fn civil_from_days(z: i64) -> (u16, u8, u8) {
    let mut z = z;
    z -= 60; // shift epoch for algorithm
    let era = (z >= 0).then_some(z).unwrap_or(z - 146096) / 146097;
    let doe = z - era * 146097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = mp + (if mp < 10 { 3 } else { -9 }); // [1, 12]
    ((y + (m <= 2) as i64) as u16, m as u8, d as u8)
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
            error!("Failed to configure DS3231: {:?}", e);
            error!("DS3231 configuration failed");
            FEEDBACK_STATE.signal(feedback::FeedbackState::Error);
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
