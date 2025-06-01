use std::env;

use log::LevelFilter;
use simplelog::{ConfigBuilder, SimpleLogger};

pub fn setup_logger() {
    let log_level = env::var("LOG_LEVEL")
        .ok()
        .and_then(|level| level.parse::<LevelFilter>().ok())
        .unwrap_or({
            if cfg!(debug_assertions) {
                LevelFilter::Debug
            } else {
                LevelFilter::Warn
            }
        });

    let config = ConfigBuilder::new()
        .set_target_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .build();

    let _ = SimpleLogger::init(log_level, config);
}
