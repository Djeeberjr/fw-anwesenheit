use embassy_time::{Duration, Timer};
use esp_hal::peripherals;
use log::{debug, error, info};

use crate::init;


#[embassy_executor::task]
pub async fn feedback_task(buzzer: peripherals::GPIO19<'static>) {
    info!("Starting feedback task");
    let mut buzzer = init::hardware::setup_buzzer(buzzer).await;
    loop {
        debug!("Buzzer feedback task running");
        buzzer.set_high();
        Timer::after(Duration::from_millis(100)).await;
        buzzer.set_low();
        Timer::after(Duration::from_millis(100)).await;
        return ;
    }
}
