//! PMS5003 UART reader and diagnostic task.
//!
//! Wiring: VCC -> 5 V, GND -> shared GND, sensor TX -> LP_GPIO4 (GPIO4),
//! sensor RX -> LP_GPIO5 (GPIO5). SET and RESET can be left unconnected.
//!
//! TEMP FILE, AI generated to test sensor

use core::cell::Cell;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use embassy_time::{Duration, Instant, Timer};
use esp_hal::peripherals::{GPIO4, GPIO5, LP_UART};
use esp_hal::uart::lp_uart::{Config as UartConfig, LpUart};
use esp_println::println;

/// Mass concentrations in µg/m³ (the "atmospheric environment" values).
#[derive(Debug, Clone, Copy)]
pub struct Pm {
    pub pm1_0: u16,
    pub pm2_5: u16,
    pub pm10: u16,
}

/// The fan needs about 30 s after wake before readings are stable.
const WARM_UP: Duration = Duration::from_secs(30);
/// Active mode sends a frame every 0.2–2.3 s, so a reading older than this means the
/// sensor stopped talking (unplugged, asleep or faulted).
const STALE_AFTER: Duration = Duration::from_secs(10);

/// Newest valid frame and when it arrived. Written by the reader task.
static LATEST: Mutex<CriticalSectionRawMutex, Cell<Option<(Pm, Instant)>>> =
    Mutex::new(Cell::new(None));

/// The newest reading, or `None` while warming up or if no frame arrived recently.
pub fn latest() -> Option<Pm> {
    LATEST
        .lock(Cell::get)
        .filter(|(_, at)| at.elapsed() < STALE_AFTER)
        .map(|(pm, _)| pm)
}

/// Configure LP_UART at 9600 8N1 and start continuous PMS5003 reception.
/// Call once, after starting the RTOS. GPIO4/5 are its fixed RX/TX pins.
pub fn start(spawner: Spawner, uart: LP_UART<'static>, rx: GPIO4<'static>, tx: GPIO5<'static>) {
    let uart = LpUart::new(uart, UartConfig::default().with_baudrate(9_600), tx, rx);
    spawner.spawn(run(uart).expect("Failed to create PMS5003 task"));
}

// esp-hal 1.2.2 exposes LP_UART configuration but no FIFO I/O methods.
// These helpers access its registers only while this task owns the LpUart.
// Polling yields to Embassy; LP_UART is not shared with the LP core.
async fn send_command(_uart: &mut LpUart, command: u8, value: u8) -> bool {
    let checksum = 0x42_u16 + 0x4d + u16::from(command) + u16::from(value);
    let [high, low] = checksum.to_be_bytes();
    let packet = [0x42, 0x4d, command, 0, value, high, low];
    let result = embassy_time::with_timeout(Duration::from_millis(500), async {
        let regs = LP_UART::regs();
        for byte in packet {
            // Waiting for an empty FIFO avoids depending on its capacity.
            while regs.status().read().txfifo_cnt().bits() != 0 {
                Timer::after(Duration::from_millis(1)).await;
            }
            regs.fifo()
                .write(|w| unsafe { w.rxfifo_rd_byte().bits(byte) });
        }
        // FIFO-empty alone does not mean the last stop bit has left the pin.
        while regs.status().read().txfifo_cnt().bits() != 0
            || regs.fsm_status().read().st_utx_out().bits() != 0
        {
            Timer::after(Duration::from_millis(1)).await;
        }
    })
    .await;
    if result.is_err() {
        println!(
            "PMS5003 command 0x{:02x}: LP_UART transmit timeout",
            command
        );
    }
    result.is_ok()
}

#[derive(Default)]
struct RxErrors {
    framing: u64,
    parity: u64,
    overflow: u64,
}

fn read_fifo(_uart: &mut LpUart, incoming: &mut [u8], errors: &mut RxErrors) -> (usize, bool) {
    let regs = LP_UART::regs();
    let flags = regs.int_raw().read();
    let error = flags.frm_err().bit_is_set()
        || flags.parity_err().bit_is_set()
        || flags.rxfifo_ovf().bit_is_set();
    errors.framing += u64::from(flags.frm_err().bit_is_set());
    errors.parity += u64::from(flags.parity_err().bit_is_set());
    errors.overflow += u64::from(flags.rxfifo_ovf().bit_is_set());
    if error {
        regs.int_clr().write(|w| {
            w.frm_err()
                .clear_bit_by_one()
                .parity_err()
                .clear_bit_by_one()
                .rxfifo_ovf()
                .clear_bit_by_one()
        });
    }
    // Drain only the count observed before this batch. This avoids relying on
    // an immediately updated cross-clock-domain count after every FIFO pop.
    // New arrivals are handled on the next poll. This is a diagnostic change,
    // not a confirmed workaround for a silicon erratum.
    let count = incoming
        .len()
        .min(regs.status().read().rxfifo_cnt().bits() as usize);
    for byte in &mut incoming[..count] {
        *byte = regs.fifo().read().rxfifo_rd_byte().bits();
    }
    (count, error)
}

#[embassy_executor::task]
async fn run(mut uart: LpUart) {
    println!("PMS5003: LP_UART RX=LP_GPIO4 (GPIO4) TX=LP_GPIO5 (GPIO5), 9600 8N1");
    let regs = LP_UART::regs();
    println!(
        "PMS5003 LP_UART config: conf0=0x{:08x} clk_conf=0x{:08x} clkdiv=0x{:08x}",
        regs.conf0().read().bits(),
        regs.clk_conf().read().bits(),
        regs.clkdiv().read().bits()
    );
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
    let mut rx_errors = RxErrors::default();
    let mut last_poll = Instant::now();
    let mut max_poll_gap_us = 0;
    // Discard any FIFO backlog/errors accumulated during startup delays and
    // command transmission, so subsequent errors represent steady-state RX.
    uart.change_baud(&UartConfig::default().with_baudrate(9_600));
    regs.int_clr().write(|w| {
        w.frm_err()
            .clear_bit_by_one()
            .parity_err()
            .clear_bit_by_one()
            .rxfifo_ovf()
            .clear_bit_by_one()
    });
    let mut bad_checksums = 0_u64;
    let mut first_bytes = [0_u8; 16];
    let mut first_count = 0;
    loop {
        let mut incoming = [0_u8; 64];
        max_poll_gap_us = max_poll_gap_us.max(last_poll.elapsed().as_micros());
        last_poll = Instant::now();
        let (count, error) = read_fifo(&mut uart, &mut incoming, &mut rx_errors);
        if error {
            errors += 1;
            used = 0;
            println!("PMS5003 LP_UART framing/parity/overflow error; resynchronizing");
        }
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
                    let pm = Pm {
                        pm1_0: u16::from_be_bytes([frame[10], frame[11]]),
                        pm2_5: u16::from_be_bytes([frame[12], frame[13]]),
                        pm10: u16::from_be_bytes([frame[14], frame[15]]),
                    };
                    let warming_up = started.elapsed() < WARM_UP;
                    println!(
                        "PMS5003 OK{}: PM1.0={} PM2.5={} PM10={} ug/m3",
                        if warming_up { " (warming up)" } else { "" },
                        pm.pm1_0,
                        pm.pm2_5,
                        pm.pm10
                    );
                    // Keep warm-up values off the display; it shows "Waiting" meanwhile
                    if !warming_up {
                        LATEST.lock(|latest| latest.set(Some((pm, Instant::now()))));
                    }
                    used = 0;
                    continue;
                }
                bad_checksums += 1;
                println!(
                    "PMS5003 checksum mismatch: calculated=0x{:04x} received=0x{:04x}",
                    sum,
                    u16::from_be_bytes([frame[30], frame[31]])
                );
            }
            // Shift one byte to recover from noise, partial frames and
            // command acknowledgments without assuming frame alignment.
            frame.copy_within(1.., 0);
            used = frame.len() - 1;
        }
        if report_at.elapsed() >= Duration::from_secs(5) {
            println!(
                "PMS5003 LP_UART errors: framing={} parity={} overflow={} max_poll_gap_us={}",
                rx_errors.framing, rx_errors.parity, rx_errors.overflow, max_poll_gap_us
            );
            println!(
                "PMS5003 totals: bytes={} valid_frames={} UART_errors={} bad_checksums={}",
                bytes, frames, errors, bad_checksums
            );
            if frames == 0 {
                println!("PMS5003 first bytes: {:02x?}", &first_bytes[..first_count]);
                println!(
                    "Check 5 V at sensor, fan running, common GND, TX -> LP_GPIO4, RX -> LP_GPIO5"
                );
            }
            report_at = Instant::now();
        }
        Timer::after(Duration::from_millis(1)).await;
    }
}
