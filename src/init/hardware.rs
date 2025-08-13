use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_time::Duration;
use embedded_sdmmc_dev::SdCard;
use esp_hal::i2c::master::Config;
use esp_hal::peripherals::{
    self, GPIO0, GPIO1, GPIO2, GPIO10, GPIO16, GPIO17, GPIO18, GPIO19, GPIO20, GPIO21, GPIO22,
    GPIO23, I2C0, RMT, SPI2, UART1,
};
use esp_hal::rmt::{ConstChannelAccess, Rmt, Tx};
use esp_hal::spi::{
    Mode,
    master::{Config as Spi_config, Spi},
};

use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{
    Async,
    clock::CpuClock,
    gpio::{Output, OutputConfig},
    i2c::master::I2c,
    timer::systimer::SystemTimer,
    uart::Uart,
};

use esp_hal_smartled::{SmartLedsAdapterAsync, buffer_size_async};

use smart_leds::colors::{BLUE, GREEN, RED};
use smart_leds::{
    RGB8, SmartLedsWriteAsync, brightness, gamma,
    hsv::{Hsv, hsv2rgb},
};

use esp_println::logger::init_logger;
use log::{debug, error};

use crate::init::network;
use crate::init::wifi;

/*************************************************
 * GPIO Pinout Xiao Esp32c6
 *
 * D0  -> GPIO0  -> Level Shifter OE
 * D1  -> GPIO1  -> Level Shifter A0 -> LED
 * D2  -> GPIO2  -> SPI/CS
 * D3  -> GPIO21 -> Buzzer
 * D4  -> GPIO22 -> I2C/SDA
 * D5  -> GPIO23 -> I2C/SCL
 * D6  -> GPIO16 -> UART/TX
 * D7  -> GPIO17 -> UART/RX -> Level Shifter A1 -> NFC Reader
 * D8  -> GPIO19 -> SPI/SCLK
 * D9  -> GPIO20 -> SPI/MISO
 * D10 -> GPIO10 -> SPI/MOSI
 *
 *************************************************/

pub const NUM_LEDS: usize  = 66;
pub const LED_BUFFER_SIZE: usize = NUM_LEDS * 25;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {
        error!("PANIC: {info}");
    }
}

esp_bootloader_esp_idf::esp_app_desc!();

pub async fn hardware_init(
    spawner: &mut Spawner,
) -> (
    Uart<'static, Async>,
    Stack<'static>,
    I2c<'static, Async>,
    SmartLedsAdapterAsync<ConstChannelAccess<esp_hal::rmt::Tx, 0>, LED_BUFFER_SIZE>,
    GPIO21<'static>,
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

    let uart_device = setup_uart(peripherals.UART1, peripherals.GPIO16, peripherals.GPIO17);

    let i2c_device = setup_i2c(peripherals.I2C0, peripherals.GPIO22, peripherals.GPIO23);

    let spi_device = setup_spi(
        peripherals.SPI2,
        peripherals.GPIO19,
        peripherals.GPIO20,
        peripherals.GPIO18,
        peripherals.GPIO2,
    );

    let sd_card = setup_sdcard(spi_device);

    let buzzer_gpio = peripherals.GPIO21;

    let mut led = setup_led(peripherals.RMT, peripherals.GPIO1);

    debug!("hardware init done");

    (uart_device, stack, i2c_device, led, buzzer_gpio)
}

// Initialize the level shifter for the NFC reader and LED (output-enable (OE) input is low, all outputs are placed in the high-impedance (Hi-Z) state)
fn init_lvl_shifter(oe_pin: GPIO0<'static>) {
    let mut oe_lvl_shifter =
        Output::new(oe_pin, esp_hal::gpio::Level::Low, OutputConfig::default());
    oe_lvl_shifter.set_high();
}

fn setup_uart(
    uart1: UART1<'static>,
    uart_tx: GPIO16<'static>,
    uart_rx: GPIO17<'static>,
) -> Uart<'static, Async> {
    let uard_device = Uart::new(uart1, esp_hal::uart::Config::default().with_baudrate(9600));

    match uard_device {
        Ok(block) => block.with_rx(uart_rx).with_tx(uart_tx).into_async(),
        Err(e) => {
            error!("Failed to initialize UART: {e}");
            panic!(); //TODO panic!
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
            panic!(); //TODO panic!
        }
    };
    i2c
}

fn setup_spi(
    spi2: SPI2<'static>,
    sck: GPIO19<'static>,
    miso: GPIO20<'static>,
    mosi: GPIO18<'static>,
    cs: GPIO2<'static>,
) -> Spi<'static, Async> {
    let spi = match Spi::new(spi2, Spi_config::default()) {
        Ok(spi) => spi
            .with_sck(sck)
            .with_miso(miso)
            .with_mosi(mosi)
            .with_cs(cs)
            .into_async(),
        Err(e) => panic!("Failed to initialize SPI: {:?}", e),
    };
    spi
}

fn setup_sdcard(spi_device: Spi<'static, Async>) {
    //let sdcard = SdCard::new(spi_device as embedded_hal::spi::SpiDevice(), delayer)
}

pub fn setup_buzzer(buzzer_gpio: GPIO21<'static>) -> Output<'static> {
    let config = esp_hal::gpio::OutputConfig::default()
        .with_drive_strength(esp_hal::gpio::DriveStrength::_40mA);
    let buzzer = Output::new(buzzer_gpio, esp_hal::gpio::Level::Low, config);

    buzzer
}

fn setup_led(
    rmt: RMT<'static>,
    led_gpio: GPIO1<'static>,
) -> SmartLedsAdapterAsync<ConstChannelAccess<esp_hal::rmt::Tx, 0>, LED_BUFFER_SIZE> {
    debug!("setup led");
    let rmt: Rmt<'_, esp_hal::Async> = {
        let frequency: Rate = Rate::from_mhz(80);
        Rmt::new(rmt, frequency)
    }
    .expect("Failed to initialize RMT")
    .into_async();

    let rmt_channel = rmt.channel0;
    let rmt_buffer = [0_u32; buffer_size_async(NUM_LEDS)];

    let led: SmartLedsAdapterAsync<_, LED_BUFFER_SIZE> =
        SmartLedsAdapterAsync::new(rmt_channel, led_gpio, rmt_buffer);

    led
}
