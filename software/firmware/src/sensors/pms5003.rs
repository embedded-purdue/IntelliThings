//! PMS5003 UART reader and diagnostic task.
//!
//! Wiring: VCC -> 5 V, GND -> shared GND, sensor TX -> GPIO20,
//! sensor RX -> GPIO21. SET and RESET can be left unconnected.
//! 
//! TEMP FILE, AI generated to test sensor

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use embedded_io_async::Write;
use esp_hal::peripherals::{GPIO20, GPIO21, UART1};
use esp_hal::uart::{Config as UartConfig, Uart};
use esp_println::println;

/// Configure UART1 at 9600 8N1 and start continuous PMS5003 reception.
/// Call once, after starting the RTOS.
pub fn start(spawner: Spawner, uart: UART1<'static>, rx: GPIO20<'static>, tx: GPIO21<'static>) {
    let uart = Uart::new(uart, UartConfig::default().with_baudrate(9_600))
        .expect("Failed to initialize PMS5003 UART1")
        .with_rx(rx)
        .with_tx(tx)
        .into_async();
    spawner.spawn(run(uart).expect("Failed to create PMS5003 task"));
}

#[embassy_executor::task]
async fn run(mut uart: Uart<'static, esp_hal::Async>) {
    async fn send_command(
        uart: &mut Uart<'static, esp_hal::Async>,
        command: u8,
        value: u8,
    ) -> bool {
        let checksum = 0x42_u16 + 0x4d + u16::from(command) + u16::from(value);
        let [high, low] = checksum.to_be_bytes();
        let packet = [0x42, 0x4d, command, 0, value, high, low];
        let result = embassy_time::with_timeout(Duration::from_millis(500), async {
            uart.write_all(&packet).await?;
            Write::flush(uart).await
        })
        .await;
        match result {
            Ok(Ok(())) => true,
            other => {
                println!("PMS5003 command 0x{:02x} failed: {:?}", command, other);
                false
            }
        }
    }

    println!("PMS5003: UART1 RX=GPIO20 TX=GPIO21, 9600 8N1");
    Timer::after(Duration::from_secs(1)).await;
    let wake_sent = send_command(&mut uart, 0xe4, 1).await;
    Timer::after(Duration::from_millis(100)).await;
    let active_sent = send_command(&mut uart, 0xe1, 1).await;
    println!(
        "PMS5003 commands transmitted: wake={} active={}; allow 30 s to stabilize",
        wake_sent, active_sent
    );

    let started = Instant::now();
    let mut report_at = Instant::now();
    let mut frame = [0_u8; 32];
    let mut used = 0;
    let mut bytes = 0_u64;
    let mut frames = 0_u64;
    let mut errors = 0_u64;
    let mut bad_checksums = 0_u64;
    let mut first_bytes = [0_u8; 16];
    let mut first_count = 0;
    loop {
        let mut incoming = [0_u8; 64];
        let count = match embassy_time::with_timeout(
            Duration::from_millis(100),
            uart.read_async(&mut incoming),
        )
        .await
        {
            Ok(Ok(count)) => count,
            Ok(Err(err)) => {
                errors += 1;
                used = 0;
                println!("PMS5003 UART error: {:?}", err);
                0
            }
            Err(_) => 0,
        };
        for &byte in &incoming[..count] {
            bytes += 1;
            if first_count < first_bytes.len() {
                first_bytes[first_count] = byte;
                first_count += 1;
            }
            frame[used] = byte;
            used += 1;
            if used < frame.len() {
                continue;
            }
            if frame[..4] == [0x42, 0x4d, 0, 28] {
                // Log all 32 bytes before validation, including checksum failures.
                println!("PMS5003 raw frame: {:02x?}", frame);
                let sum: u16 = frame[..30].iter().map(|&b| u16::from(b)).sum();
                if sum == u16::from_be_bytes([frame[30], frame[31]]) {
                    frames += 1;
                    println!(
                        "PMS5003 OK{}: PM1.0={} PM2.5={} PM10={} ug/m3",
                        if started.elapsed() < Duration::from_secs(30) {
                            " (warming up)"
                        } else {
                            ""
                        },
                        u16::from_be_bytes([frame[10], frame[11]]),
                        u16::from_be_bytes([frame[12], frame[13]]),
                        u16::from_be_bytes([frame[14], frame[15]])
                    );
                    used = 0;
                    continue;
                }
                bad_checksums += 1;
            }
            // Shift one byte to recover from noise, partial frames and
            // command acknowledgments without assuming frame alignment.
            frame.copy_within(1.., 0);
            used = frame.len() - 1;
        }
        if report_at.elapsed() >= Duration::from_secs(5) {
            println!(
                "PMS5003 totals: bytes={} valid_frames={} UART_errors={} bad_checksums={}",
                bytes, frames, errors, bad_checksums
            );
            if frames == 0 {
                println!("PMS5003 first bytes: {:02x?}", &first_bytes[..first_count]);
                println!(
                    "Check 5 V at sensor, fan running, common GND, TX -> GPIO20, RX -> GPIO21"
                );
            }
            report_at = Instant::now();
        }
        Timer::after(Duration::from_millis(1)).await;
    }
}
