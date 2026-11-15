//! Light bar demo: cycles through the light-bar patterns that already exist, without Wi-Fi,
//! sensors or the display, to check the strip and pattern switching.
//!
//! Flash with `cargo run --release --bin light_bar_demo`; the log names each pattern as it starts.
//! Wiring is the same as the main firmware: GPIO10 -> level shifter -> strip DIN, shared GND.

#![no_std]
#![no_main]

use companion_ui::lightbar::{self, Pattern, Rgb};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{clock::CpuClock, rmt::PulseCode, timer::timg::TimerGroup};
use esp_println::println;
use intelli_things::{
    led::{self, Led},
    light_bar,
};
use static_cell::ConstStaticCell;

/// Pixels on the strip; keep in step with `LED_PIXELS` in main.rs.
const LED_PIXELS: usize = 160;
static LED_FRAME: ConstStaticCell<[PulseCode; led::frame_len(LED_PIXELS)]> =
    ConstStaticCell::new([led::BLANK; led::frame_len(LED_PIXELS)]);

esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Light bar demo panic: {}", info);
    loop {
        core::hint::spin_loop();
    }
}

extern crate alloc;

/// Air-quality states the Home screen can produce: calm glow when good, meter otherwise.
/// The meter fractions match what `director::pattern_for` picks (CO2 ppm / 2000).
const AIR: [(&str, Pattern, u64); 6] = [
    ("Air good: calm glow", Pattern::Calm(lightbar::GOOD), 8),
    ("Air moderate: meter (CO2 ~1000 ppm)", Pattern::Meter(0.5), 4),
    ("Air poor: meter (CO2 ~1300 ppm)", Pattern::Meter(0.66), 4),
    ("Air bad: meter (CO2 ~1800 ppm)", Pattern::Meter(0.9), 4),
    ("Room segments: good / moderate / bad", ROOMS, 6),
    ("Alert: pulse", Pattern::Alert, 5),
];

const ROOMS: Pattern = Pattern::Rooms {
    colours: [lightbar::GOOD, lightbar::WARN, lightbar::BAD, lightbar::GOOD],
    count: 3,
    // the bad room throbs
    pulse: 0b100,
};

/// The agent, presence and status patterns.
const OTHERS: [(&str, Pattern, u64); 6] = [
    ("Unread statement: breathe", Pattern::Unread, 6),
    ("Agent thinking: comet", Pattern::Thinking, 6),
    ("Confirm: warm sweep", Pattern::Confirm, 6),
    ("Greeting: flare", Pattern::Greeting, 5),
    ("Night light: dim centre", Pattern::Night, 5),
    ("Hub offline: status pixel", Pattern::Offline, 5),
];

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(size: 8 * 1024);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let led = Led::new(peripherals.RMT, peripherals.GPIO10, LED_FRAME.take())
        .expect("LED strip start");
    spawner.spawn(light_bar::task(led).expect("light bar task"));
    println!(
        "Light bar demo: {} pixels, brightness cap {}",
        LED_PIXELS,
        light_bar::BRIGHTNESS_CAP
    );

    loop {
        // A smooth sweep shows the meter's green -> amber -> red bands, like CO2 rising
        println!("Air meter sweep: 0% -> 100%");
        for step in 0..=50 {
            light_bar::set_pattern(Pattern::Meter(step as f32 / 50.0), 0.0);
            Timer::after(Duration::from_millis(80)).await;
        }
        show_all(&AIR).await;
        show_all(&OTHERS).await;
        println!("Off");
        light_bar::set_pattern(Pattern::Off, 0.0);
        Timer::after(Duration::from_secs(2)).await;
    }
}

/// Shows each pattern from the start of its animation for its number of seconds.
async fn show_all(patterns: &[(&str, Pattern, u64)]) {
    for &(name, pattern, secs) in patterns {
        println!("{} (~{} mA)", name, peak_ma(&pattern, secs));
        light_bar::set_pattern(pattern, 0.0);
        Timer::after(Duration::from_secs(secs)).await;
    }
}

/// Rough peak strip current for a pattern over its run, from the engine's estimate.
fn peak_ma(pattern: &Pattern, secs: u64) -> u32 {
    let mut frame = [Rgb::default(); LED_PIXELS];
    (0..secs * 4)
        .map(|q| {
            lightbar::render(pattern, q as f32 / 4.0, light_bar::BRIGHTNESS_CAP, &mut frame);
            lightbar::estimate_ma(&frame)
        })
        .max()
        .unwrap_or(0)
}
