#![no_std]
#![no_main]

use esp_hal::{
    Blocking,
    clock::CpuClock,
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    rmt::{Channel, PulseCode, Rmt, Tx, TxChannelConfig, TxChannelCreator},
    time::Rate,
};
use esp_println::println;
use static_cell::StaticCell;

// Wiring: GPIO10 -> level shifter -> strip DIN; ESP32 and strip share GND.
// Supply the strip with external 5 V. Connect at DIN (follow the strip arrows).
// Set this to the number of addressable pixels, which can differ from LED count.
const LED_COUNT: usize = 160;
const BRIGHTNESS: u8 = 50;
// Set false to restore the normal LED strip test after measuring GPIO10.
const GPIO10_HIGH_TEST: bool = false;
const FRAME_LEN: usize = LED_COUNT * 24 + 2;

// Keep the ~15 KB waveform off the stack.
static FRAME: StaticCell<[PulseCode; FRAME_LEN]> = StaticCell::new();

// RMT source = 80 MHz, divider = 2: each tick is 25 ns.
// WS2812B datasheet nominal timing: 0 = 400/850 ns, 1 = 800/450 ns.
// Both symbols are 1.25 us (800 kbit/s).
const ZERO: PulseCode = PulseCode::new(Level::High, 16, Level::Low, 34);
const ONE: PulseCode = PulseCode::new(Level::High, 32, Level::Low, 18);
// Explicit 800 us LOW latch/reset, followed by the hardware end marker.
const RESET: PulseCode = PulseCode::new(Level::Low, 16_000, Level::Low, 16_000);

esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("LED test panic: {}", info);
    loop {
        core::hint::spin_loop();
    }
}

fn encode(frame: &mut [PulseCode; FRAME_LEN], mut pixel: impl FnMut(usize) -> [u8; 3]) {
    for led in 0..LED_COUNT {
        let [red, green, blue] = pixel(led);
        for (component, value) in [green, red, blue].into_iter().enumerate() {
            for bit in 0..8 {
                frame[led * 24 + component * 8 + bit] = if value & (0x80 >> bit) != 0 {
                    ONE
                } else {
                    ZERO
                };
            }
        }
    }
    frame[FRAME_LEN - 2] = RESET;
    frame[FRAME_LEN - 1] = PulseCode::end_marker();
}

fn send<'a>(channel: Channel<'a, Blocking, Tx>, frame: &[PulseCode]) -> Channel<'a, Blocking, Tx> {
    // Blocking RMT refills its hardware RAM while waiting. Keep interrupts from
    // delaying those refills. This standalone test does not run Wi-Fi/Embassy.
    let result = critical_section::with(|_| match channel.transmit(frame) {
        Ok(transaction) => transaction.wait(),
        Err(error) => Err(error),
    });
    match result {
        Ok(channel) => channel,
        Err((error, channel)) => {
            println!("RMT transmission failed: {:?}", error);
            channel
        }
    }
}

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    let delay = Delay::new();
    if GPIO10_HIGH_TEST {
        let _pin = Output::new(peripherals.GPIO10, Level::High, OutputConfig::default());
        loop {
            println!("GPIO10 held HIGH: measure GPIO10 to ESP32 GND (about 3.3 V)");
            delay.delay_millis(1_000);
        }
    }
    println!(
        "LED strip test starting: GPIO10, {} pixels, brightness {}/255",
        LED_COUNT, BRIGHTNESS
    );

    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).unwrap();
    let config = TxChannelConfig::default()
        .with_clk_divider(2)
        // Reserve both TX memory blocks for channel 0, doubling refill headroom.
        .with_memsize(2)
        .with_idle_output(true)
        .with_idle_output_level(Level::Low);
    let mut channel = rmt
        .channel0
        .configure_tx(&config)
        .unwrap()
        .with_pin(peripherals.GPIO10);
    let frame = FRAME.init_with(|| [PulseCode::end_marker(); FRAME_LEN]);

    // Let the strip power settle while DIN is held low, then clear every pixel.
    delay.delay_millis(500);
    encode(frame, |_| [0, 0, 0]);
    channel = send(channel, frame);
    delay.delay_millis(100);

    loop {
        for (name, color) in [
            ("red", [1, 0, 0]),
            ("green", [0, 1, 0]),
            ("blue", [0, 0, 1]),
            ("off", [0, 0, 0]),
        ] {
            println!("LED strip: {}", name);
            for bright in 0..BRIGHTNESS {
                let temp_color = 
                    [color[0] * bright, color[1] * bright, color[2] * bright];
                encode(frame, |_| temp_color);
                channel = send(channel, frame);
                delay.delay_millis(25);
            }
        }

        println!("LED strip: moving red pixel (first to last)");
        for position in 0..LED_COUNT {
            encode(frame, |led| {
                if led == position {
                    [BRIGHTNESS, 0, 0]
                } else {
                    [0, 0, 0]
                }
            });
            channel = send(channel, frame);
            delay.delay_millis(25);
        }
    }
}
