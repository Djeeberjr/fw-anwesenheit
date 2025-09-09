use core::cell::RefCell;

use bleps::att::Att;
use critical_section::Mutex;
use ds3231::InterruptControl;
use embassy_executor::Spawner;
use embassy_net::Stack;

use embassy_time::{Duration, Timer};
use esp_hal::gpio::{Input, InputConfig, Io};
use esp_hal::i2c::master::Config;
use esp_hal::interrupt::InterruptHandler;
use esp_hal::peripherals::{
    GPIO0, GPIO1, GPIO16, GPIO17, GPIO18, GPIO19, GPIO20, GPIO21, GPIO22, GPIO23, I2C0, RMT, SPI2,
    UART1,
};
use esp_hal::rmt::{ConstChannelAccess, Rmt};
use esp_hal::spi::master::{Config as Spi_config, Spi};

use esp_hal::{Blocking, handler};
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
use esp_hal_embassy::InterruptExecutor;
use esp_hal_smartled::{SmartLedsAdapterAsync, buffer_size_async};
use esp_println::dbg;
use esp_println::logger::init_logger;
use log::{debug, error, info};

use crate::FEEDBACK_STATE;
use crate::init::network;
use crate::init::sd_card::setup_sdcard;
use crate::init::wifi;
use crate::store::AttendanceDay;
use crate::store::persistence::Persistence;

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

pub const NUM_LEDS: usize = 66;
pub const LED_BUFFER_SIZE: usize = NUM_LEDS * 25;

static SD_DET: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));

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

    Timer::after(Duration::from_millis(1)).await;

    let uart_device = setup_uart(peripherals.UART1, peripherals.GPIO16, peripherals.GPIO17);

    let i2c_device = setup_i2c(peripherals.I2C0, peripherals.GPIO22, peripherals.GPIO23);

    let mut io = Io::new(peripherals.IO_MUX);
    io.set_interrupt_handler(handler);
    let mut sd_det = Input::new(peripherals.GPIO0, InputConfig::default());
    critical_section::with(|cs| {
    // Here we are listening for a low level to demonstrate
    // that you need to stop listening for level interrupts,
    // but usually you'd probably use `FallingEdge`.
    sd_det.listen(esp_hal::gpio::Event::AnyEdge);
    SD_DET.borrow_ref_mut(cs).replace(sd_det);
});


    let spi_bus = setup_spi(
        peripherals.SPI2,
        peripherals.GPIO19,
        peripherals.GPIO20,
        peripherals.GPIO18,
    );

    let sd_cs_pin = Output::new(
        peripherals.GPIO2,
        esp_hal::gpio::Level::High,
        OutputConfig::default(),
    );

    let mut vol_mgr = setup_sdcard(spi_bus, sd_cs_pin);

    let buzzer_gpio = peripherals.GPIO21;

    Timer::after(Duration::from_millis(500)).await;

    let led = setup_led(peripherals.RMT, peripherals.GPIO1);

    debug!("hardware init done");

    (uart_device, stack, i2c_device, led, buzzer_gpio)
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
) -> Spi<'static, Blocking> {
    let spi = match Spi::new(spi2, Spi_config::default()) {
        Ok(spi) => spi.with_sck(sck).with_miso(miso).with_mosi(mosi),
        Err(e) => panic!("Failed to initialize SPI: {:?}", e),
    };
    spi
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


#[handler]
fn handler() {
    critical_section::with(|cs| {
        let mut sd_det = SD_DET.borrow_ref_mut(cs);
        let Some(sd_det) = sd_det.as_mut() else {
            // Some other interrupt has occurred
            // before the button was set up.
            return;
        };

        if sd_det.is_interrupt_set() {
            //card is insert on high 
            if  sd_det.is_high() {
                debug!("card insert");
                //FEEDBACK_STATE.signal(crate::feedback::FeedbackState::Ack);
            //sd_det.unlisten();
                //sd_det.listen(esp_hal::gpio::Event::FallingEdge);
                
            }
            //card is not insert on low
            else  {
                debug!("card removed");
                //FEEDBACK_STATE.signal(crate::feedback::FeedbackState::Nack);
                //sd_det.unlisten();
                sd_det.listen(esp_hal::gpio::Event::RisingEdge);
            }
        }
    });
}