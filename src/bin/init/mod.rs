use embassy_executor::Spawner;
use embassy_net::Stack;
use esp_hal::peripherals::{GPIO1, GPIO2, UART1};
use esp_hal::{
    Async,
    clock::CpuClock,
    timer::{systimer::SystemTimer, timg::TimerGroup},
    uart::Uart,
};
use esp_println::logger::init_logger;
use log::error;

mod network;
mod wifi;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

pub async fn hardware_init(spawner: &mut Spawner) -> (Uart<'static, Async>, Stack<'static>) {
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

    let uart_devie = setup_uart(peripherals.UART1, peripherals.GPIO1, peripherals.GPIO2);

    (uart_devie, stack)
}

fn setup_uart(
    uart1: UART1<'static>,
    gpio1: GPIO1<'static>,
    gpio2: GPIO2<'static>,
) -> Uart<'static, Async> {
    let uard_device = Uart::new(uart1, esp_hal::uart::Config::default().with_baudrate(9600));

    match uard_device {
        Ok(block) => block.with_rx(gpio1).with_tx(gpio2).into_async(),
        Err(e) => {
            error!("Failed to initialize UART: {e}");
            panic!();
        }
    }
}
