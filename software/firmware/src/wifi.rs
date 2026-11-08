use embassy_executor::Spawner;
use embassy_net::icmp::PacketMetadata;
use embassy_net::icmp::ping::{PingManager, PingParams};
use embassy_net::tcp::TcpSocket;
use embassy_net::{Config as NetConfig, Runner, Stack, StackResources};
use embassy_time::{Duration, Timer, with_timeout};
use esp_hal::peripherals::WIFI;
use esp_println::println;
use esp_radio::wifi::{
    AuthenticationMethodConfig, Config as WifiConfig, ConnectionError, Interface, WifiController,
    sta::StationConfig,
};
use static_cell::StaticCell;

const RETRY_DELAY: Duration = Duration::from_secs(5);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);

// Network resources for DHCP, diagnostics, and MQTT TCP.
// DHCP and DNS each keep a socket occupied; the ping test, then SNTP, need a third, MQTT needs a fourth.
static NET_RESOURCES: StaticCell<StackResources<4>> = StaticCell::new(); 

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
    let mut rx_buffer = [0u8; 1024];
    let mut tx_buffer = [0u8; 1024];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

    socket.set_timeout(Some(Duration::from_secs(10)));

    println!("Testing connection to MQTT broker...");

    match socket
        .connect((embassy_net::Ipv4Address::new(192, 168, 1, 2), 1883))
        .await
    {
        Ok(()) => {
            println!("Success: ESP32 reached MQTT broker port 1883");
        }
        Err(error) => {
            println!("Failed to reach MQTT broker: {:?}", error);
        }
    }
}

async fn reboot_for_wifi_retry() -> ! {
    println!("Restarting ESP32 to retry Wi-Fi in 5 seconds");
    Timer::after(RETRY_DELAY).await;
    esp_hal::system::software_reset()
}

/// Retries Wi-Fi association and runs a ping test if DHCP completes within 20 seconds.
/// DHCP continues in the background after a timeout. Keep the returned controller
/// alive for as long as the connection is needed; check stack readiness before use.
pub async fn setup<'d>(spawner: Spawner, wifi: WIFI<'d>) -> (WifiController<'d>, Stack<'static>) {
    let wifi_ssid = option_env!("WIFI_SSID").unwrap_or("");
    let wifi_password = option_env!("WIFI_PASSWORD").unwrap_or("");
    let (ssid, password) = loop {
        if !wifi_ssid.is_empty()
            && !wifi_password.is_empty()
            && let (Ok(ssid), Ok(password)) = (wifi_ssid.try_into(), wifi_password.try_into())
        {
            break (ssid, password);
        }
        println!("Set valid WIFI_SSID and WIFI_PASSWORD in .cargo/esp-config.toml and reflash");
        Timer::after(RETRY_DELAY).await;
    };

    let mut wifi_controller = match WifiController::new(wifi, Default::default()) {
        Ok(controller) => controller,
        Err(error) => {
            println!("Failed to initialize Wi-Fi controller: {:?}", error);
            reboot_for_wifi_retry().await;
        }
    };
    let wifi_config = WifiConfig::Station(
        StationConfig::default()
            .with_ssid(ssid)
            .with_authentication(AuthenticationMethodConfig::Wpa2Personal(password)),
    );
    if let Err(error) = wifi_controller.set_config(&wifi_config) {
        println!("Failed to apply Wi-Fi configuration: {:?}", error);
        reboot_for_wifi_retry().await;
    }

    loop {
        println!("Connecting to Wi-Fi...");
        match with_timeout(CONNECT_TIMEOUT, wifi_controller.connect_async()).await {
            Ok(Ok(_)) => break,
            Ok(Err(ConnectionError::Failed(info))) => {
                println!(
                    "Wi-Fi connection failed: {:?}. Retrying in 5 seconds",
                    info.reason
                );
                Timer::after(RETRY_DELAY).await;
            }
            Ok(Err(error)) => {
                println!("Wi-Fi driver error while connecting: {:?}", error);
                reboot_for_wifi_retry().await;
            }
            Err(_) => {
                // Dropping the future stops waiting for an event, but the radio may
                // still be trying to connect. Restart before beginning another attempt.
                println!("Wi-Fi connection attempt timed out");
                reboot_for_wifi_retry().await;
            }
        }
    }
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

    println!("Waiting for a DHCP address (up to 20 seconds)...");
    if with_timeout(Duration::from_secs(20), stack.wait_config_up())
        .await
        .is_err()
    {
        println!("DHCP timed out; starting sensors while DHCP continues in the background");
        return (wifi_controller, stack);
    }

    println!("DHCP configured");
    ping_smoke_test(stack).await;

    (wifi_controller, stack)
}
