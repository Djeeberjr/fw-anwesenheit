use core::slice::RChunks;

use ds3231::{Alarm1Config, DS3231Error, DS3231};
use embassy_executor::Spawner;
use embassy_net::{driver, Stack};
use embassy_sync::mutex::Mutex;
use esp_hal::config;
use esp_hal::gpio::{Input, Pull};
use esp_hal::i2c::master::Config;
use esp_hal::peripherals::{self, GPIO0, GPIO1, GPIO3, GPIO4, GPIO5, GPIO6, GPIO7, GPIO21, GPIO22, GPIO23, I2C0, UART1};
use esp_hal::time::Rate;
use esp_hal::{
    Async,
    clock::CpuClock,
    timer::{systimer::SystemTimer, timg::TimerGroup},
    uart::Uart,
    i2c::master::I2c,
    gpio::{Output, OutputConfig}
};
use esp_println::logger::init_logger;
use log::error;

use crate::init::wifi;
use crate::init::network;

const RTC_ADDRESS: u8 =  0x57;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

pub async fn hardware_init(spawner: &mut Spawner) -> (Uart<'static, Async>, Stack<'static>, I2c<'static, Async>, GPIO21<'static>) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    init_logger(log::LevelFilter::Debug);

    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let mut rng = esp_hal::rng::Rng::new(peripherals.RNG);
    let network_seed = (rng.random() as u64) << 32 | rng.random() as u64;

    wifi::set_antenna_mode(peripherals.GPIO3, peripherals.GPIO14).await;
    let interfaces = wifi::setup_wifi(timer1.timer0, rng, peripherals.WIFI, spawner);
    let stack = network::setup_network(network_seed, interfaces.ap, spawner);

    init_lvl_shifter(peripherals.GPIO0);

    let uart_device = setup_uart(peripherals.UART1, peripherals.GPIO7, peripherals.GPIO6);

    let i2c_device = setup_i2c(peripherals.I2C0, peripherals.GPIO22, peripherals.GPIO23);

    //RTC Interrupt pin
    let sqw_pin = peripherals.GPIO21;

    //TODO change to get I2C device back / maybe init for each protocol

    (uart_device, stack, i2c_device, sqw_pin)
}

// Initialize the level shifter for the NFC reader and LED (output-enable (OE) input is low, all outputs are placed in the high-impedance (Hi-Z) state)
fn init_lvl_shifter(oe_pin: GPIO0<'static>){
    let mut oe_lvl_shifter = Output::new(oe_pin, esp_hal::gpio::Level::Low, OutputConfig::default());
    oe_lvl_shifter.set_high();
}

fn setup_uart(
    uart1: UART1<'static>,
    uart_rx: GPIO7<'static>,
    uart_tx: GPIO6<'static>,
) -> Uart<'static, Async> {
    let uard_device = Uart::new(uart1, esp_hal::uart::Config::default().with_baudrate(9600));

    match uard_device {
        Ok(block) => block.with_rx(uart_rx).with_tx(uart_tx).into_async(),
        Err(e) => {
            error!("Failed to initialize UART: {e}");
            panic!();
        }
    }
}

fn setup_i2c(
    i2c0: I2C0<'static>,
    sda: GPIO22<'static>,
    scl: GPIO23<'static>,
) -> I2c<'static, Async> {

    let config = Config::default().with_frequency(Rate::from_khz(400));
    let i2c = match I2c::new(i2c0, config) {
        Ok(i2c) => i2c.with_sda(sda).with_scl(scl).into_async(),
        Err(e) => {
            error!("Failed to initialize I2C: {:?}", e);
            panic!();
        }
    };
    i2c
}

pub async fn rtc_init_iterrupt(sqw_pin: GPIO21<'static>) -> Input<'static> {
    let config = esp_hal::gpio::InputConfig::default().with_pull(Pull::Up); //Active low interrupt in rtc
    let sqw_interrupt = Input::new(sqw_pin, config);
    sqw_interrupt
}

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


fn setup_spi_led() {

}