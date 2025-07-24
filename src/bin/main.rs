#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

use core::net::Ipv4Addr;
use core::str::FromStr;

use embassy_executor::Spawner;
use embassy_net::{Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4};
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Output, OutputConfig};
use esp_hal::peripherals::{GPIO1, GPIO2, UART1};
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::uart::{Config, Uart};
use esp_println::logger::init_logger;
use esp_wifi::wifi::{
    AccessPointConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState,
};
use log::{debug, info};
use picoserve::routing::get;
use picoserve::{AppBuilder, AppRouter};
use static_cell::make_static;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // ------------------- init ---------------------------
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    info!("starting up...");

    esp_alloc::heap_allocator!(size: 72 * 1024);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    init_logger(log::LevelFilter::Debug);

    let timer1 = TimerGroup::new(peripherals.TIMG0);
    let mut rng = esp_hal::rng::Rng::new(peripherals.RNG);

    debug!("set wlan antenna..");
    let mut rf_switch = Output::new(
        peripherals.GPIO3,
        esp_hal::gpio::Level::Low,
        OutputConfig::default(),
    );

    rf_switch.set_low();

    Timer::after_secs(1).await;

    let mut antenna_mode = Output::new(
        peripherals.GPIO14,
        esp_hal::gpio::Level::Low,
        OutputConfig::default(),
    );

    antenna_mode.set_low();

    Timer::after_secs(1).await;

    // Setup wifi deivce
    debug!("setup wifi..");
    let esp_wifi_ctrl =
        make_static!(esp_wifi::init(timer1.timer0, rng).unwrap());
    let (controller, interfaces) = esp_wifi::wifi::new(esp_wifi_ctrl, peripherals.WIFI).unwrap();
    // let wifi_interface = interfaces.sta;
    let wifi_ap = interfaces.ap;

    let gw_ip_addr_str = "192.168.2.1";
    let gw_ip_addr = Ipv4Addr::from_str(gw_ip_addr_str).expect("failed to parse gateway ip");

    let config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(gw_ip_addr, 24),
        gateway: Some(gw_ip_addr),
        dns_servers: Default::default(),
    });

    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    let (stack, runner) = embassy_net::new(
        wifi_ap,
        config,
        make_static!(StackResources::<3>::new()),
        seed,
    );

    debug!("Setup complete. Running network tasks");

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(runner)).ok();
    spawner.spawn(run_dhcp(stack, gw_ip_addr_str)).ok();
    spawner
        .spawn(rfid_reader_task(
            peripherals.UART1,
            peripherals.GPIO1,
            peripherals.GPIO2,
        ))
        .ok();

    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
        if stack.is_config_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    debug!("Starting webserver");

    let app = make_static!(AppProps.build_app());

    let config = make_static!(picoserve::Config::new(picoserve::Timeouts {
        start_read_request: Some(Duration::from_secs(5)),
        persistent_start_read_request: Some(Duration::from_secs(1)),
        read_request: Some(Duration::from_secs(1)),
        write: Some(Duration::from_secs(1)),
    }));

    let _ = spawner.spawn(webserver_task(0, stack, app, config));
}

struct AppProps;

impl AppBuilder for AppProps {
    type PathRouter = impl picoserve::routing::PathRouter;

    fn build_app(self) -> picoserve::Router<Self::PathRouter> {
        picoserve::Router::new().route("/", get(|| async move { "Hello World" }))
    }
}

#[embassy_executor::task]
async fn webserver_task(
    id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config<Duration>,
) -> ! {
    let mut tcp_rx_buffer = [0u8; 1024];
    let mut tcp_tx_buffer = [0u8; 1024];
    let mut http_buffer = [0u8; 2048];

    picoserve::listen_and_serve(
        id,
        app,
        config,
        stack,
        80,
        &mut tcp_rx_buffer,
        &mut tcp_tx_buffer,
        &mut http_buffer,
    )
    .await
}

#[embassy_executor::task]
async fn run_dhcp(stack: Stack<'static>, gw_ip_addr: &'static str) {
    debug!("start dhcp");
    use core::net::{Ipv4Addr, SocketAddrV4};

    use edge_dhcp::{
        io::{self, DEFAULT_SERVER_PORT},
        server::{Server, ServerOptions},
    };
    use edge_nal::UdpBind;
    use edge_nal_embassy::{Udp, UdpBuffers};

    let ip = Ipv4Addr::from_str(gw_ip_addr).expect("dhcp task failed to parse gw ip");

    let mut buf = [0u8; 1500];

    let mut gw_buf = [Ipv4Addr::UNSPECIFIED];

    let buffers = UdpBuffers::<3, 1024, 1024, 10>::new();
    let unbound_socket = Udp::new(stack, &buffers);
    let mut bound_socket = unbound_socket
        .bind(core::net::SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            DEFAULT_SERVER_PORT,
        )))
        .await
        .unwrap();

    loop {
        _ = io::server::run(
            &mut Server::<_, 64>::new_with_et(ip),
            &ServerOptions::new(ip, Some(&mut gw_buf)),
            &mut bound_socket,
            &mut buf,
        )
        .await
        .inspect_err(|e| log::warn!("DHCP server error: {e:?}"));
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await;
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    debug!("start connection task");
    debug!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::ApStarted => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::ApStop).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::AccessPoint(AccessPointConfiguration {
                ssid: "esp-wifi".try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            debug!("Starting wifi");
            controller.start_async().await.unwrap();
            debug!("Wifi started!");
        }
    }
}

#[embassy_executor::task]
async fn rfid_reader_task(uart1: UART1<'static>, gpio1: GPIO1<'static>, gpio2: GPIO2<'static>) {
    debug!("init rfid reader..");

    let uart1_block_result = Uart::new(uart1, Config::default().with_baudrate(9600));
    let mut nfc_reader = match uart1_block_result {
        Ok(block) => block.with_rx(gpio1).with_tx(gpio2).into_async(),
        Err(e) => {
            log::error!("Failed to initialize UART: {:?}", e);
            return;
        }
    };

    let mut uart_buffer = [0u8; 64];

    loop {
        debug!("Looking for NFC...");
        match nfc_reader.read_async(&mut uart_buffer).await {
            Ok(n) => {
                let mut hex_str = heapless::String::<128>::new();
                for byte in &uart_buffer[..n] {
                    core::fmt::Write::write_fmt(&mut hex_str, format_args!("{:02X} ", byte)).ok();
                }
                info!("Read {} bytes from UART: {}", n, hex_str);
            }
            Err(e) => {
                log::error!("Error reading from UART: {:?}", e);
            }
        }
        Timer::after(Duration::from_millis(200)).await;
    }
}
