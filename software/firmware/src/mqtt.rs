//! One-shot MQTT 5 publishing smoke test using the existing Embassy stack.

use embassy_net::{Ipv4Address, Stack, tcp::TcpSocket};
use embassy_time::{Duration, Timer, with_timeout};
use esp_println::println;
use rust_mqtt::{
    Bytes,
    buffer::AllocBuffer,
    client::{
        Client,
        event::Event,
        options::{ConnectOptions, DisconnectOptions, PublicationOptions, TopicReference},
    },
    types::{MqttString, TopicName},
};

const BROKER: Ipv4Address = Ipv4Address::new(192, 168, 1, 2);
const PORT: u16 = 1883;
const TOPIC: &str = "intellithings/esp32c6/telemetry";
const MESSAGE: &[u8] = b"Hello from IntelliThings ESP32-C6!";

/// Publishes one fixed message at QoS 1, then exits after broker acknowledgement.
/// Failed attempts retry after five seconds; an uncertain acknowledgement can
/// cause a duplicate message. No additional network runner is created.
#[embassy_executor::task]
pub async fn publish_task(stack: Stack<'static>) {
    loop {
        stack.wait_config_up().await;
        // Outer error is timeout, inner error is test failure
        match with_timeout(Duration::from_secs(20), publish_once(stack)).await {
            Ok(Ok(())) => return,
            Ok(Err(error)) => println!("MQTT test failed: {}", error),
            Err(_) => println!("MQTT test timed out"),
        }
        Timer::after(Duration::from_secs(5)).await;
    }
}

async fn publish_once(stack: Stack<'static>) -> Result<(), &'static str> {
    let mut rx = [0; 1024];
    let mut tx = [0; 1024];
    let mut socket = TcpSocket::new(stack, &mut rx, &mut tx);
    socket.set_timeout(Some(Duration::from_secs(10)));
    println!("MQTT: connecting to {}:{}", BROKER, PORT);
    socket
        .connect((BROKER, PORT))
        .await
        .map_err(|error| {
            println!("MQTT: TCP connect error: {:?}", error);
            "TCP connection failed"
        })?;

    let mut buffer = AllocBuffer;
    let mut client = Client::<'_, '_, _, _, 1, 1, 1, 0, 1, 0, 0>::new(&mut buffer);
    // Let the broker assign a unique client ID for this short-lived test.
    client
        .connect(socket, &ConnectOptions::new().clean_start(), None)
        .await
        .map_err(|_| "MQTT CONNECT failed (requires an MQTT 5 broker)")?;

    let topic = TopicName::new(MqttString::from_str(TOPIC)
        .map_err(|_| "Topic name error")?).ok_or("Topic name error")?;
    let options = PublicationOptions::new(TopicReference::Name(topic)).at_least_once();
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
    println!("MQTT: broker acknowledged message on {}", TOPIC);
    let mut socket = client
        .disconnect(&DisconnectOptions::new())
        .await
        .map_err(|_| "MQTT DISCONNECT failed")?;
    socket.close();
    socket.flush().await.map_err(|_| "TCP close failed")?;
    Ok(())
}
