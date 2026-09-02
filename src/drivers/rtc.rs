use chrono::{TimeZone, Utc};
use ds3231::{
    Config, DS3231, InterruptControl, Oscillator, SquareWaveFrequency, TimeRepresentation,
};
use esp_hal::{
    Async,
    i2c::{self, master::I2c},
};
use log::{debug, error, info};

use crate::{FEEDBACK_STATE, drivers, feedback};

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
            Ok(datetime) => datetime.and_utc().timestamp() as u64,
            Err(e) => {
                FEEDBACK_STATE.signal(feedback::FeedbackState::Error);
                error!("Failed to read RTC datetime: {:?}", e);
                0
            }
        }
    }

    pub async fn set_time(&mut self, time: u64) {
        let naive_dt = Utc
            .timestamp_opt(time as i64, 0)
            .single()
            .expect("create native datetime")
            .naive_utc();

        self.dev
            .set_datetime(&naive_dt)
            .await
            .expect("Set datetime");
        info!("Set rtc to {:?}", naive_dt)
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
        oscillator_enable: Oscillator::Enabled,
    };

    match rtc.configure(&rtc_config).await {
        Ok(_) => info!("DS3231 configured successfully"),
        Err(e) => {
            error!("Failed to configure DS3231: {:?}", e);
            error!("DS3231 configuration failed");
            FEEDBACK_STATE.signal(feedback::FeedbackState::Error);
        }
    }

    if rtc.datetime().await.unwrap() < naive_dt {
        rtc.set_datetime(&naive_dt).await.unwrap_or_else(|e| {
            FEEDBACK_STATE.signal(feedback::FeedbackState::Error);
            error!("Failed to set RTC datetime: {:?}", e);
        });
        info!("RTC datetime set to: {}", naive_dt);
    }

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

    info!("RTC time is: {:?}", rtc.datetime().await.unwrap());

    rtc
}
