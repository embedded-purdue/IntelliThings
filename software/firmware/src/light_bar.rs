//! Light bar: animates the WS2812B strip with `companion-ui`'s pattern engine.
//!
//! The display task decides which pattern fits the current state (air quality, greeting,
//! night, ...) and hands it over with [`set_pattern`]. This task renders that pattern about
//! 30 times a second, so animations stay smooth even though the state changes slowly.

use core::cell::Cell;

use companion_ui::lightbar::{self, Pattern, Rgb};
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use embassy_time::{Duration, Instant, Timer};
use esp_println::println;

use crate::led::{self, Led};

/// Brightness ceiling, 0–1. Every pattern is scaled by this, which also limits the strip's
/// current draw. The companion node uses 0.35 for its 24 LEDs.
pub const BRIGHTNESS_CAP: f32 = 0.35;

/// About 30 frames a second.
const FRAME_MS: u64 = 33;

/// The pattern to show and when (ms since boot) it started; its animation time counts from there.
static CURRENT: Mutex<CriticalSectionRawMutex, Cell<(Pattern, u64)>> =
    Mutex::new(Cell::new((Pattern::Off, 0)));

/// Switches the light bar to `pattern`, `elapsed_s` seconds into its animation.
///
/// Call it as often as you like: passing the same pattern and elapsed time each frame
/// keeps the animation running smoothly.
pub fn set_pattern(pattern: Pattern, elapsed_s: f32) {
    let now = Instant::now().as_millis();
    let start = now.saturating_sub((elapsed_s.max(0.0) * 1000.0) as u64);
    CURRENT.lock(|c| c.set((pattern, start)));
}

/// The pattern being shown now.
pub fn current() -> Pattern {
    CURRENT.lock(|c| c.get().0)
}

/// Renders the current pattern onto the strip forever.
#[embassy_executor::task]
pub async fn task(mut led: Led<'static>) {
    let mut frame = [Rgb::default(); led::PIXELS];
    let mut reported = false;
    loop {
        let (pattern, start) = CURRENT.lock(|c| c.get());
        let t = Instant::now().as_millis().saturating_sub(start) as f32 / 1000.0;
        lightbar::render(&pattern, t, BRIGHTNESS_CAP, &mut frame);
        match led.show(&frame).await {
            Ok(()) => reported = false,
            // Report a failure once, not 30 times a second
            Err(err) if !reported => {
                println!("Light bar: RMT error {:?}", err);
                reported = true;
            }
            Err(_) => {}
        }
        Timer::after(Duration::from_millis(FRAME_MS)).await;
    }
}
