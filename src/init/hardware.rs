use embassy_executor::Spawner;
use embassy_net::{Stack, driver};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use esp_hal::config;
use esp_hal::gpio::{Input, Pull};
use esp_hal::i2c::master::Config;
use esp_hal::peripherals::{
    self, GPIO0, GPIO1, GPIO3, GPIO4, GPIO5, GPIO6, GPIO7, GPIO19, GPIO21, GPIO22, GPIO23, I2C0,
    UART1,
};
use esp_hal::time::Rate;
use esp_hal::{
    Async,
    clock::CpuClock,
    gpio::{Output, OutputConfig},
    i2c::master::I2c,
    timer::{systimer::SystemTimer, timg::TimerGroup},
    uart::Uart,
};
use esp_println::logger::init_logger;
use log::{debug, error};

use crate::init::network;
use crate::init::wifi;

/*************************************************
 * GPIO Pinout Xiao Esp32c6
 *
 * D0 -> GPIO0  -> Level Shifter OE
 * D1 -> GPIO1  -> Level Shifter A0 -> LED
 * D3 -> GPIO21 -> SQW Interrupt RTC //not in use anymore
 * D4 -> GPIO22 -> SDA
 * D5 -> GPIO23 -> SCL
 * D7 -> GPIO17 -> Level Shifter A1 -> NFC Reader
 * D8 -> GPIO19 -> Buzzer
 *
 *************************************************/

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

pub async fn hardware_init(
    spawner: &mut Spawner,
) -> (
    Uart<'static, Async>,
    Stack<'static>,
    I2c<'static, Async>,
    GPIO19<'static>,
) {
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

    let buzzer_gpio = peripherals.GPIO19;

    debug!("hardware init done");

    (uart_device, stack, i2c_device, buzzer_gpio)
}

// Initialize the level shifter for the NFC reader and LED (output-enable (OE) input is low, all outputs are placed in the high-impedance (Hi-Z) state)
fn init_lvl_shifter(oe_pin: GPIO0<'static>) {
    let mut oe_lvl_shifter =
        Output::new(oe_pin, esp_hal::gpio::Level::Low, OutputConfig::default());
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
    debug!("init I2C");
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

pub async fn setup_rtc_iterrupt(sqw_pin: GPIO21<'static>) -> Input<'static> {
    debug!("init rtc interrupt");
    let config = esp_hal::gpio::InputConfig::default().with_pull(Pull::Up); //Active low interrupt in rtc
    let sqw_interrupt = Input::new(sqw_pin, config);
    sqw_interrupt
}

pub fn setup_buzzer(buzzer_gpio: GPIO19<'static>) -> Output<'static> {
    let config = esp_hal::gpio::OutputConfig::default()
        .with_drive_strength(esp_hal::gpio::DriveStrength::_40mA);
    let buzzer = Output::new(buzzer_gpio, esp_hal::gpio::Level::Low, config);

    buzzer
}

fn setup_spi_led() {}
