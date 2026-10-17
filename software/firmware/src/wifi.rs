use embassy_executor::Spawner;
use embassy_net::icmp::PacketMetadata;
use embassy_net::icmp::ping::{PingManager, PingParams};
use embassy_net::{Config as NetConfig, Runner, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::peripherals::WIFI;
use esp_println::println;
use esp_radio::wifi::{
    AuthenticationMethodConfig, Config as WifiConfig, Interface, WifiController, sta::StationConfig,
};
use static_cell::StaticCell;

// DHCP keeps one socket occupied; the ping test needs a second socket.
static NET_RESOURCES: StaticCell<StackResources<2>> = StaticCell::new();
static ICMP_RX_META: StaticCell<[PacketMetadata; 1]> = StaticCell::new();
static ICMP_TX_META: StaticCell<[PacketMetadata; 1]> = StaticCell::new();
static ICMP_RX_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
static ICMP_TX_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();

#[embassy_executor::task]
async fn network_task(mut runner: Runner<'static, Interface>) -> ! {
    runner.run().await
}

async fn ping_smoke_test(stack: Stack<'static>) {
    println!("Pinging 8.8.8.8...");
    let mut ping_manager = PingManager::new(
        stack,
        ICMP_RX_META.init([PacketMetadata::EMPTY]),
        ICMP_RX_BUFFER.init([0; 256]),
        ICMP_TX_META.init([PacketMetadata::EMPTY]),
        ICMP_TX_BUFFER.init([0; 256]),
    );
    let target = core::net::Ipv4Addr::new(8, 8, 8, 8);
    let mut params = PingParams::new(target);
    params
        .set_count(1)
        .set_timeout(Duration::from_secs(3))
        .set_rate_limit(Duration::from_millis(100));

    match ping_manager.ping(&params).await {
        Ok(round_trip_time) => println!("Ping 8.8.8.8: {} ms", round_trip_time.as_millis()),
        Err(error) => println!("Ping 8.8.8.8 failed: {:?}", error),
    }
}

pub async fn network_call(stack: Stack<'static>) {
    
}

/// Connects to Wi-Fi and runs a ping test. Keep the returned controller alive
/// for as long as the connection is needed.
pub async fn setup<'d>(spawner: Spawner, wifi: WIFI<'d>) -> (WifiController<'d>, Stack<'static>) {
    let wifi_ssid = option_env!("WIFI_SSID").unwrap_or("");
    let wifi_password = option_env!("WIFI_PASSWORD").unwrap_or("");
    if wifi_ssid.is_empty() || wifi_password.is_empty() {
        println!("Set WIFI_SSID and WIFI_PASSWORD in .cargo/esp-config.toml");
        loop {
            Timer::after(Duration::from_secs(5)).await;
        }
    }

    let mut wifi_controller = WifiController::new(wifi, Default::default())
        .expect("Failed to initialize Wi-Fi controller");
    let wifi_config = WifiConfig::Station(
        StationConfig::default()
            .with_ssid(wifi_ssid.try_into().expect("WIFI_SSID is invalid"))
            .with_authentication(AuthenticationMethodConfig::Wpa2Personal(
                wifi_password.try_into().expect("WIFI_PASSWORD is invalid"),
            )),
    );
    wifi_controller
        .set_config(&wifi_config)
        .expect("Failed to apply Wi-Fi configuration");
    wifi_controller
        .connect_async()
        .await
        .expect("Failed to connect to Wi-Fi");
    println!("Wi-Fi connected");

    let interface = Interface::station();
    let resources = NET_RESOURCES.init(StackResources::new());
    let (stack, runner) = embassy_net::new(
        interface,
        NetConfig::dhcpv4(Default::default()),
        resources,
        0x1,
    );
    spawner.spawn(network_task(runner).expect("Failed to prepare network task"));

    while !stack.is_config_up() {
        Timer::after(Duration::from_millis(100)).await;
    }
    println!("DHCP configured");
    ping_smoke_test(stack).await;

    (wifi_controller, stack)
}
