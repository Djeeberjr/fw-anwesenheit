
pub async fn rtc_config(i2c: I2c<'static, Async>) -> DS3231<I2c<'static, Async>> {
    let mut rtc: DS3231<I2c<'static, Async>> = DS3231::new(i2c, RTC_ADDRESS);
    let daily_alarm = Alarm1Config::AtTime {
        hours: 0,   // set alarm every day 00:00:00 to sync time
        minutes: 0,
        seconds: 0,
        is_pm: None, // 24-hour mode
    };
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