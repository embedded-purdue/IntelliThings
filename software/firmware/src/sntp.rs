//! Network time: asks Google's SNTP server for the time and hands it to the display clock.
//! A minimal SNTPv4 client (RFC 4330): one 48-byte UDP request, the server's transmit time back.

use embassy_net::dns::DnsQueryType;
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{IpEndpoint, Stack};
use embassy_time::{Duration, Instant, Timer, with_timeout};
use esp_println::println;

use crate::display;

const SERVER: &str = "time.google.com";
const PORT: u16 = 123;
/// Seconds from the NTP epoch (1900) to the Unix epoch (1970)
const NTP_TO_UNIX: u32 = 2_208_988_800;
const PACKET_LEN: usize = 48;
const REPLY_TIMEOUT: Duration = Duration::from_secs(5);

/// Local offset from UTC in minutes, for the display clock. Fixed, so it doesn't follow
/// daylight saving: Eastern is -300 (EST, November to March) and -240 (EDT, March to November).
const UTC_OFFSET_MIN: i16 = -300;
/// The display keeps time between syncs; this only corrects drift.
const RESYNC_EVERY: Duration = Duration::from_secs(60 * 60);
const RETRY_AFTER: Duration = Duration::from_secs(30);

/// Keeps the display clock set from SNTP. Waits for the network to be up before each query.
#[embassy_executor::task]
pub async fn task(stack: Stack<'static>) {
    loop {
        stack.wait_config_up().await;
        match query(stack).await {
            Ok(unix) => {
                println!("SNTP: {} gave Unix time {}", SERVER, unix);
                display::set_clock(unix, UTC_OFFSET_MIN);
                Timer::after(RESYNC_EVERY).await;
            }
            Err(err) => {
                println!("SNTP error: {}; retrying in {} s", err, RETRY_AFTER.as_secs());
                Timer::after(RETRY_AFTER).await;
            }
        }
    }
}

/// Asks the server once and returns the current Unix time in seconds.
async fn query(stack: Stack<'static>) -> Result<u32, &'static str> {
    let addresses = stack
        .dns_query(SERVER, DnsQueryType::A)
        .await
        .map_err(|_| "DNS lookup failed")?;
    let server = IpEndpoint::new(*addresses.first().ok_or("DNS returned no address")?, PORT);

    let mut rx_meta = [PacketMetadata::EMPTY; 1];
    let mut rx_buffer = [0u8; 64];
    let mut tx_meta = [PacketMetadata::EMPTY; 1];
    let mut tx_buffer = [0u8; 64];
    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
    // Port 0 picks a free local port
    socket.bind(0).map_err(|_| "UDP bind failed")?;

    // LI 0 (no warning), version 4, mode 3 (client). The transmit timestamp is only a nonce:
    // the server echoes it as the originate timestamp, which ties the reply to this request.
    let mut request = [0u8; PACKET_LEN];
    request[0] = 0x23;
    let nonce = Instant::now().as_micros().to_be_bytes();
    request[40..48].copy_from_slice(&nonce);
    socket.send_to(&request, server).await.map_err(|_| "UDP send failed")?;

    let mut reply = [0u8; PACKET_LEN];
    loop {
        let (len, meta) = with_timeout(REPLY_TIMEOUT, socket.recv_from(&mut reply))
            .await
            .map_err(|_| "no reply from the server")?
            .map_err(|_| "UDP receive failed")?;
        // Skip anything that isn't the server's answer to this request
        if meta.endpoint != server || len < PACKET_LEN || reply[24..32] != nonce {
            continue;
        }
        return unix_seconds(&reply);
    }
}

/// Checks a server reply and converts its transmit time to Unix seconds.
fn unix_seconds(reply: &[u8; PACKET_LEN]) -> Result<u32, &'static str> {
    let leap = reply[0] >> 6;
    let mode = reply[0] & 0x07;
    let stratum = reply[1];
    if mode != 4 {
        return Err("reply isn't from a server");
    }
    // Stratum 0 is a "kiss-o'-death" (e.g. rate limited); LI 3 means the server isn't synced
    if stratum == 0 || leap == 3 {
        return Err("server refused or isn't synchronised");
    }
    let ntp_seconds = u32::from_be_bytes([reply[40], reply[41], reply[42], reply[43]]);
    // Valid until NTP's era rolls over in 2036
    ntp_seconds
        .checked_sub(NTP_TO_UNIX)
        .ok_or("server time is before 1970")
}
