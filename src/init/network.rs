use core::net::{Ipv4Addr, SocketAddrV4};
use edge_dhcp::{
    io::{self, DEFAULT_SERVER_PORT},
    server::{Server, ServerOptions},
};
use edge_nal::UdpBind;
use edge_nal_embassy::{Udp, UdpBuffers};
use embassy_executor::Spawner;
use embassy_net::{Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4};
use embassy_time::{Duration, Timer};
use esp_radio::wifi::WifiDevice;
use static_cell::StaticCell;

use crate::webserver::WEB_TAKS_SIZE;

pub const NETWORK_STACK_SIZE: usize = WEB_TAKS_SIZE + 2; // + 2 for other network taks. Breaks without
pub const GW_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 2, 1);

pub fn setup_network<'a>(seed: u64, wifi: WifiDevice<'static>, spawner: Spawner) -> Stack<'a> {
    let config = embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(GW_IP, 24),
        gateway: Some(GW_IP),
        dns_servers: Default::default(),
    });

    static NETWORK_STACK: StaticCell<StackResources<NETWORK_STACK_SIZE>> = StaticCell::new();
    let nw_stack = NETWORK_STACK.init(StackResources::new());

    let (stack, runner) = embassy_net::new(wifi, config, nw_stack, seed);

    spawner.must_spawn(net_task(runner));
    spawner.must_spawn(run_dhcp(stack));

    stack
}

#[embassy_executor::task]
async fn run_dhcp(stack: Stack<'static>) {
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
        .expect("Failed to bind socket for DHCP server");

    loop {
        _ = io::server::run(
            &mut Server::<_, 64>::new_with_et(GW_IP),
            &ServerOptions::new(GW_IP, Some(&mut gw_buf)),
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
