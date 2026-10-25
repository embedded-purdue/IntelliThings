//! LD2410C presence sensor smoke test, using UART1 at the factory 256000 baud.
//! Sensor TX -> GPIO20, RX -> GPIO21, VCC -> 5 V, GND -> shared ground.
//! OUT is unused. Disconnect the PMS5003 signal wires before using this test.
//! Protocol: Hi-Link LD2410C Serial Communication Protocol V1.00, section 2.3.

#[cfg(not(test))]
pub use hardware::start;

#[derive(Debug, PartialEq)]
struct Reading {
    state: u8,
    moving_cm: u16,
    moving_energy: u8,
    stationary_cm: u16,
    stationary_energy: u8,
    detection_cm: u16,
}

// A bounded streaming parser: UART reads may split or combine reports.
struct Parser {
    buffer: [u8; 64],
    used: usize,
    invalid_frames: u64,
}

impl Parser {
    fn new() -> Self {
        Self {
            buffer: [0; 64],
            used: 0,
            invalid_frames: 0,
        }
    }

    fn discard(&mut self, count: usize) {
        self.buffer.copy_within(count..self.used, 0);
        self.used -= count;
    }

    fn push(&mut self, byte: u8) -> Option<Reading> {
        if self.used == self.buffer.len() {
            self.discard(1);
        }
        self.buffer[self.used] = byte;
        self.used += 1;
        loop {
            if self.used < 4 {
                return None;
            }
            if self.buffer[..4] != [0xf4, 0xf3, 0xf2, 0xf1] {
                self.discard(1);
                continue;
            }
            if self.used < 6 {
                return None;
            }
            let length = usize::from(u16::from_le_bytes([self.buffer[4], self.buffer[5]]));
            if !(13..=self.buffer.len() - 10).contains(&length) {
                self.invalid_frames += 1;
                self.discard(1);
                continue;
            }
            let total = length + 10;
            if self.used < total {
                return None;
            }
            let data = &self.buffer[..total];
            // Basic reports have 13 payload bytes; engineering reports include
            // the same basic fields plus gate energies and optional extra fields.
            let supported = (data[6] == 2 && length == 13) || (data[6] == 1 && length >= 35);
            if supported
                && data[7] == 0xaa
                && data[8] <= 3
                && data[total - 6..total - 4] == [0x55, 0x00]
                && data[total - 4..] == [0xf8, 0xf7, 0xf6, 0xf5]
            {
                let reading = Reading {
                    state: data[8],
                    moving_cm: u16::from_le_bytes([data[9], data[10]]),
                    moving_energy: data[11],
                    stationary_cm: u16::from_le_bytes([data[12], data[13]]),
                    stationary_energy: data[14],
                    detection_cm: u16::from_le_bytes([data[15], data[16]]),
                };
                self.discard(total);
                return Some(reading);
            }
            self.invalid_frames += 1;
            self.discard(1);
        }
    }
}

#[cfg(not(test))]
mod hardware {
    use super::Parser;
    use embassy_executor::Spawner;
    use embassy_time::{Duration, Instant, with_timeout};
    use esp_hal::{
        Async,
        peripherals::{GPIO20, GPIO21, UART1},
        uart::{Config, Uart},
    };
    use esp_println::println;

    /// Start once after RTOS initialization. The sensor must use its default baud rate.
    pub fn start(spawner: Spawner, uart: UART1<'static>, rx: GPIO20<'static>, tx: GPIO21<'static>) {
        let uart = Uart::new(uart, Config::default().with_baudrate(256_000))
            .expect("Failed to initialize LD2410C UART1")
            .with_rx(rx)
            .with_tx(tx)
            .into_async();
        spawner.spawn(run(uart).expect("Failed to create LD2410C test task"));
    }

    #[embassy_executor::task]
    async fn run(mut uart: Uart<'static, Async>) {
        println!("LD2410C: UART1 RX=GPIO20 TX=GPIO21, 256000 8N1; waiting for reports");
        let mut parser = Parser::new();
        let mut bytes = 0_u64;
        let mut frames = 0_u64;
        let mut errors = 0_u64;
        let mut sample = [0_u8; 24];
        let mut sample_count = 0;
        let mut last_state = None;
        let mut last_print = Instant::now();
        let mut last_report = Instant::now();
        let mut last_frame = None;
        loop {
            let mut incoming = [0_u8; 64];
            let count = match with_timeout(
                Duration::from_millis(250),
                uart.read_async(&mut incoming),
            )
            .await
            {
                Ok(Ok(count)) => count,
                Ok(Err(error)) => {
                    errors += 1;
                    parser.used = 0;
                    println!("LD2410C UART error: {:?}", error);
                    0
                }
                Err(_) => {
                    // Drop incomplete frames after an idle gap, then hunt for a header.
                    parser.used = 0;
                    0
                }
            };
            for &byte in &incoming[..count] {
                bytes += 1;
                if sample_count < sample.len() {
                    sample[sample_count] = byte;
                    sample_count += 1;
                }
                if let Some(reading) = parser.push(byte) {
                    frames += 1;
                    last_frame = Some(Instant::now());
                    // Show transitions immediately; otherwise limit logging to 1 Hz.
                    if last_state != Some(reading.state)
                        || last_print.elapsed() >= Duration::from_secs(1)
                    {
                        let state = match reading.state {
                            0 => "no presence",
                            1 => "moving",
                            2 => "stationary",
                            _ => "moving + stationary",
                        };
                        println!(
                            "LD2410C OK: {} | moving={} cm energy={} | stationary={} cm energy={} | detection={} cm",
                            state,
                            reading.moving_cm,
                            reading.moving_energy,
                            reading.stationary_cm,
                            reading.stationary_energy,
                            reading.detection_cm
                        );
                        last_state = Some(reading.state);
                        last_print = Instant::now();
                    }
                }
            }
            if last_report.elapsed() >= Duration::from_secs(5) {
                println!(
                    "LD2410C totals: bytes={} valid_frames={} UART_errors={} invalid_frames={}",
                    bytes, frames, errors, parser.invalid_frames
                );
                if last_frame.is_none_or(|time| time.elapsed() >= Duration::from_secs(5)) {
                    println!(
                        "LD2410C: no recent valid reports; first bytes={:02x?}",
                        &sample[..sample_count]
                    );
                    println!(
                        "Check sensor TX -> GPIO20, RX -> GPIO21, 5 V, shared GND and baud=256000"
                    );
                }
                last_report = Instant::now();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Basic report example from the manufacturer's protocol, with distances
    // extended above 255 cm here to verify little-endian decoding.
    const BASIC: [u8; 23] = [
        0xf4, 0xf3, 0xf2, 0xf1, 13, 0, 2, 0xaa, 3, 0x2c, 1, 60, 0x90, 1, 57, 0x2c, 1, 0x55, 0,
        0xf8, 0xf7, 0xf6, 0xf5,
    ];

    #[test]
    fn decodes_fragmented_and_consecutive_reports() {
        let mut parser = Parser::new();
        for &byte in &BASIC[..10] {
            assert_eq!(parser.push(byte), None);
        }
        let result = BASIC[10..]
            .iter()
            .filter_map(|&b| parser.push(b))
            .next()
            .unwrap();
        assert_eq!(
            result,
            Reading {
                state: 3,
                moving_cm: 300,
                moving_energy: 60,
                stationary_cm: 400,
                stationary_energy: 57,
                detection_cm: 300
            }
        );
        for state in 0..=3 {
            let mut frame = BASIC;
            frame[8] = state;
            assert_eq!(
                frame
                    .iter()
                    .filter_map(|&b| parser.push(b))
                    .next()
                    .unwrap()
                    .state,
                state
            );
        }
    }

    #[test]
    fn rejects_corruption_and_recovers_from_noise() {
        for index in [4, 5, 6, 7, 8, 17, 18, 19, 22] {
            let mut parser = Parser::new();
            let mut broken = BASIC;
            broken[index] = 0xff;
            for &byte in &[0, 0xf4, 0xf3, 0] {
                assert_eq!(parser.push(byte), None);
            }
            for byte in broken {
                assert_eq!(parser.push(byte), None);
            }
            assert!(
                BASIC
                    .iter()
                    .filter_map(|&b| parser.push(b))
                    .next()
                    .is_some()
            );
            assert!(parser.invalid_frames > 0);
        }
    }

    #[test]
    fn reads_basic_fields_from_engineering_report() {
        let mut frame = [0_u8; 45];
        frame[..17].copy_from_slice(&BASIC[..17]);
        frame[4] = 35;
        frame[6] = 1;
        frame[39..].copy_from_slice(&BASIC[17..]);
        let mut parser = Parser::new();
        let result = frame.iter().filter_map(|&b| parser.push(b)).next().unwrap();
        assert_eq!(result.moving_cm, 300);
        assert_eq!(result.stationary_cm, 400);
    }
}
