use embassy_executor::Spawner;
use embassy_net::Stack;
use embassy_time::{Duration, Timer};
use esp_hal::Blocking;
use esp_hal::delay::Delay;
use esp_hal::gpio::AnyPin;
use esp_hal::i2c::master::Config;
use esp_hal::peripherals::{
    GPIO1, GPIO16, GPIO17, GPIO18, GPIO19, GPIO20, GPIO21, GPIO22, GPIO23, I2C0, RMT, SPI2, UART1,
};
use esp_hal::rmt::Rmt;
use esp_hal::spi::master::{Config as Spi_config, Spi};
use esp_hal::system::software_reset;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{
    Async,
    clock::CpuClock,
    gpio::{Output, OutputConfig},
    i2c::master::I2c,
    uart::Uart,
};
use esp_hal_smartled::{SmartLedsAdapterAsync, buffer_size_async};
use esp_println::logger::init_logger;
use log::{debug, error};
use smart_leds::SmartLedsWriteAsync;
use smart_leds::brightness;
use smart_leds::colors::RED;
use thiserror::Error;

use crate::init::network;
use crate::init::sd_card::{SDCardPersistence, setup_sdcard};
use crate::init::wifi;

/*************************************************
 * GPIO Pinout Xiao Esp32c6
 *
 * D0  -> GPIO0  -> SD DECT
 * D1  -> GPIO1  -> Level Shifter A0 -> LED
 * D2  -> GPIO2  -> SPI/CS
 * D3  -> GPIO21 -> Buzzer
 * D4  -> GPIO22 -> I2C/SDA
 * D5  -> GPIO23 -> I2C/SCL
 * D6  -> GPIO16 -> UART/TX
 * D7  -> GPIO17 -> UART/RX -> Level Shifter A1 -> NFC Reader
 * D8  -> GPIO19 -> SPI/SCLK
 * D9  -> GPIO20 -> SPI/MISO
 * D10 -> GPIO18 -> SPI/MOSI
 *
 *************************************************/

pub const NUM_LEDS: usize = 1;
pub const LED_BUFFER_SIZE: usize = buffer_size_async(NUM_LEDS);

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let delay = Delay::new();
    error!("PANIC: {info}");
    delay.delay(esp_hal::time::Duration::from_secs(30));
    software_reset()
}

esp_bootloader_esp_idf::esp_app_desc!();

#[derive(Error, Debug)]
pub enum HardwareInitError {
    #[error("Failed to etup UART")]
    Uart(#[from] esp_hal::uart::ConfigError),

    #[error("Failed to setup I2C")]
    I2C(#[from] esp_hal::i2c::master::ConfigError),

    #[error("Failed to setup SPI")]
    Spi(#[from] esp_hal::spi::master::ConfigError),

    #[error("Failed to setuo LED")]
    Led(#[from] esp_hal::rmt::Error),

    #[error("Failed to setup wifi")]
    Wifi(#[from] wifi::WifiError),
}

pub struct AppHardware {
    pub uart: Uart<'static, Async>,
    pub network_stack: Stack<'static>,
    pub i2c: I2c<'static, Async>,
    pub buzzer: Output<'static>,
    pub sd_present: AnyPin<'static>,
    pub led: SmartLedsAdapterAsync<'static, LED_BUFFER_SIZE>,
    pub sdcard: SDCardPersistence,
}

impl AppHardware {
    pub async fn init(spawner: Spawner) -> Result<Self, HardwareInitError> {
        let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
        let peripherals = esp_hal::init(config);

        esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 65536);

        let timg0 = TimerGroup::new(peripherals.TIMG0);
        let sw_interrupt =
            esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
        esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

        init_logger(log::LevelFilter::Debug);

        let mut led = setup_led(peripherals.RMT, peripherals.GPIO1)?;
        let _ = led.write(brightness(
                    [RED; NUM_LEDS].into_iter(),
                    255,
                ))
                .await;

        let rng = esp_hal::rng::Rng::new();
        let network_seed = (rng.random() as u64) << 32 | rng.random() as u64;

        wifi::set_antenna_mode(peripherals.GPIO3, peripherals.GPIO14).await;
        let interfaces = wifi::setup_wifi(peripherals.WIFI, spawner)?;
        let network_stack = network::setup_network(network_seed, interfaces.ap, spawner);

        Timer::after(Duration::from_millis(1)).await;

        let uart_device = setup_uart(peripherals.UART1, peripherals.GPIO16, peripherals.GPIO17)?;

        let i2c_device = setup_i2c(peripherals.I2C0, peripherals.GPIO22, peripherals.GPIO23)?;

        let sd_det_gpio = peripherals.GPIO0;

        let spi_bus = setup_spi(
            peripherals.SPI2,
            peripherals.GPIO19,
            peripherals.GPIO20,
            peripherals.GPIO18,
        )?;

        let sd_cs_pin = Output::new(
            peripherals.GPIO2,
            esp_hal::gpio::Level::High,
            OutputConfig::default(),
        );

        let vol_mgr = setup_sdcard(spi_bus, sd_cs_pin);

        let buzzer_gpio = peripherals.GPIO21;
        let buzzer = setup_buzzer(buzzer_gpio);

        Timer::after(Duration::from_millis(500)).await;

        debug!("hardware init done");

        Ok(Self {
            uart: uart_device,
            network_stack,
            i2c: i2c_device,
            buzzer,
            sd_present: sd_det_gpio.into(),
            led,
            sdcard: vol_mgr,
        })
    }
}

fn setup_uart(
    uart1: UART1<'static>,
    uart_tx: GPIO16<'static>,
    uart_rx: GPIO17<'static>,
) -> Result<Uart<'static, Async>, esp_hal::uart::ConfigError> {
    let uart_device = Uart::new(uart1, esp_hal::uart::Config::default().with_baudrate(9600))?;
    Ok(uart_device.with_rx(uart_rx).with_tx(uart_tx).into_async())
}

fn setup_i2c(
    i2c0: I2C0<'static>,
    sda: GPIO22<'static>,
    scl: GPIO23<'static>,
) -> Result<I2c<'static, Async>, esp_hal::i2c::master::ConfigError> {
    debug!("init I2C");
    let config = Config::default().with_frequency(Rate::from_khz(400));

    let i2c = I2c::new(i2c0, config)?;

    Ok(i2c.with_sda(sda).with_scl(scl).into_async())
}

fn setup_spi(
    spi2: SPI2<'static>,
    sck: GPIO19<'static>,
    miso: GPIO20<'static>,
    mosi: GPIO18<'static>,
) -> Result<Spi<'static, Blocking>, esp_hal::spi::master::ConfigError> {
    let spi = Spi::new(spi2, Spi_config::default())?;
    Ok(spi.with_sck(sck).with_miso(miso).with_mosi(mosi))
}

pub fn setup_buzzer(buzzer_gpio: GPIO21<'static>) -> Output<'static> {
    let config = esp_hal::gpio::OutputConfig::default()
        .with_drive_strength(esp_hal::gpio::DriveStrength::_40mA);

    Output::new(buzzer_gpio, esp_hal::gpio::Level::Low, config)
}

fn setup_led<'a>(
    rmt: RMT<'a>,
    led_gpio: GPIO1<'a>,
) -> Result<esp_hal_smartled::SmartLedsAdapterAsync<'a, LED_BUFFER_SIZE>, esp_hal::rmt::Error> {
    let rmt: Rmt<'_, esp_hal::Async> = {
        let frequency: Rate = Rate::from_mhz(80);
        Rmt::new(rmt, frequency)
    }?
    .into_async();

    let rmt_channel = rmt.channel0;
    let rmt_buffer = [esp_hal::rmt::PulseCode::default(); LED_BUFFER_SIZE];

    Ok(SmartLedsAdapterAsync::new(
        rmt_channel,
        led_gpio,
        rmt_buffer,
    ))
}
