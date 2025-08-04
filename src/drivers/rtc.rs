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
use chrono::{TimeZone, Utc};

include!(concat!(env!("OUT_DIR"), "/build_time.rs"));

const RTC_ADDRESS: u8 = 0x68;

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
