//! Periodic MQTT 5 publishing smoke test using the existing Embassy stack.

extern crate alloc;

use alloc::format;
use embassy_net::{HardwareAddress, Ipv4Address, Stack, tcp::TcpSocket};
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
const MESSAGE: &[u8] = b"Hello from IntelliThings ESP32-C6!";

/// Publishes a fixed message at QoS 1 every five seconds.
/// Reconnects after a failed connection or publish.
#[embassy_executor::task]
pub async fn publish_task(stack: Stack<'static>) {
    loop {
        stack.wait_config_up().await;
        if let Err(error) = publish_session(stack).await {
            println!("MQTT test failed: {}", error);
        }
        Timer::after(Duration::from_secs(5)).await;
    }
}

async fn publish_session(stack: Stack<'static>) -> Result<(), &'static str> {
    let HardwareAddress::Ethernet(mac) = stack.hardware_address();
    let [a, b, c, d, e, f] = mac.0;
    let topic_path =
        format!("intellithings/{a:02X}:{b:02X}:{c:02X}:{d:02X}:{e:02X}:{f:02X}/sensors");

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
    loop {
        with_timeout(Duration::from_secs(10), async {
            let packet_id = client
                .publish(&options, Bytes::from(MESSAGE))
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
