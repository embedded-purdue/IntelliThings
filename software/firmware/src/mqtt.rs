//! Periodic MQTT 5 publishing of the latest sensor snapshot.

extern crate alloc;

use crate::{readings, sntp};
use alloc::{
    format,
    string::{String, ToString},
};
use core::fmt::Write;
use embassy_net::{EthernetAddress, HardwareAddress, Ipv4Address, Stack, tcp::TcpSocket};
use embassy_time::{Duration, Timer, with_timeout};
use esp_println::println;
use rust_mqtt::{
    Bytes,
    buffer::AllocBuffer,
    client::{
        Client,
        event::Event,
        options::{ConnectOptions, PublicationOptions, TopicReference},
    },
    types::{MqttBinary, MqttString, TopicName},
};

const BROKER: Ipv4Address = Ipv4Address::new(192, 168, 1, 2);
const PORT: u16 = 1883;

/// Publishes the latest readings at QoS 1 every five seconds.
/// Reconnects after a failed connection or publish.
#[embassy_executor::task]
pub async fn publish_task(stack: Stack<'static>) {
    loop {
        stack.wait_config_up().await;
        // Wait for a real wall-clock timestamp.
        while sntp::unix_now().is_none() {
            Timer::after(Duration::from_secs(1)).await;
        }
        if let Err(error) = publish_session(stack).await {
            println!("MQTT publish failed: {}", error);
        }
        Timer::after(Duration::from_secs(5)).await;
    }
}

fn get_topic_path(mac: EthernetAddress) -> String {
    let [a, b, c, d, e, f] = mac.0;
    let mac_string = format!("{a:02X}:{b:02X}:{c:02X}:{d:02X}:{e:02X}:{f:02X}");
    format!("intellithings/{mac_string}/sensors")
}

/// Reads the synchronized clock and formats the current UTC timestamp.
fn get_timestamp() -> Result<heapless::String<32>, &'static str> {
    let unix = sntp::unix_now().ok_or("Clock has not synchronized")?;
    let time = companion_ui::clock::DateTime::from_unix(unix as i64, 0);
    let mut timestamp = heapless::String::<32>::new();
    write!(
        timestamp,
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        time.year, time.month, time.day, time.hour, time.minute, time.second
    )
    .map_err(|_| "Timestamp buffer too small")?;
    Ok(timestamp)
}

async fn publish_session(stack: Stack<'static>) -> Result<(), &'static str> {
    let HardwareAddress::Ethernet(mac) = stack.hardware_address();
    let topic_path = get_topic_path(mac);
    let node_id = mac.to_string();

    let options = ConnectOptions::new().clean_start();
    let options = match (option_env!("MQTT_USER"), option_env!("MQTT_PASSWORD")) {
        (Some(user), Some(password)) => options
            .user_name(MqttString::from_str(user).map_err(|_| "Invalid MQTT_USER")?)
            .password(MqttBinary::try_from(password).map_err(|_| "Invalid MQTT_PASSWORD")?),
        (None, None) => options,
        _ => return Err("Set both MQTT_USER and MQTT_PASSWORD in .cargo/esp-config.toml"),
    };

    let mut rx = [0; 1024];
    let mut tx = [0; 1024];
    let mut socket = TcpSocket::new(stack, &mut rx, &mut tx);
    socket.set_timeout(Some(Duration::from_secs(10)));
    println!("MQTT: connecting to {}:{}", BROKER, PORT);
    with_timeout(Duration::from_secs(10), socket.connect((BROKER, PORT)))
        .await
        .map_err(|_| "TCP connection timed out")?
        .map_err(|error| {
            println!("MQTT: TCP connect error: {:?}", error);
            "TCP connection failed"
        })?;

    let mut buffer = AllocBuffer;
    let mut client = Client::<'_, '_, _, _, 1, 1, 1, 0, 1, 0, 0>::new(&mut buffer);
    // Let the broker assign a unique client ID.
    with_timeout(
        Duration::from_secs(10),
        client.connect(socket, &options, None),
    )
    .await
    .map_err(|_| "MQTT CONNECT timed out")?
    .map_err(|_| "MQTT CONNECT failed (check credentials and MQTT 5 broker support)")?;

    let topic = TopicName::new(MqttString::from_str(&topic_path).map_err(|_| "Topic name error")?)
        .ok_or("Topic name error")?;
    let options = PublicationOptions::new(TopicReference::Name(topic)).at_least_once();
    let mut payload = [0u8; 512];
    loop {
        let readings = readings::latest();
        let timestamp = get_timestamp()?;
        let len = readings
            .to_json(&node_id, &timestamp, &mut payload)
            .map_err(|_| "JSON serialization failed")?;
        with_timeout(Duration::from_secs(10), async {
            let packet_id = client
                .publish(&options, Bytes::from(&payload[..len]))
                .await
                .map_err(|_| "PUBLISH failed")?
                .ok_or("QoS 1 publish did not return a packet ID")?;

            loop {
                match client
                    .poll()
                    .await
                    .map_err(|_| "Failed waiting for PUBACK")?
                {
                    Event::PublishAcknowledged(ack) if ack.packet_identifier == packet_id => break,
                    Event::PublishRejected(_) => return Err("Broker rejected the publication"),
                    _ => {}
                }
            }
            Ok(())
        })
        .await
        .map_err(|_| "MQTT publish timed out")??;
        println!("MQTT: broker acknowledged message on {}", topic_path);
        Timer::after(Duration::from_secs(5)).await;
    }
}
