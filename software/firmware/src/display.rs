//! DWIN display: runs the `companion-ui` drawing code on this node.
//! The main loop hands over sensor readings in the VP map's format (`docs/interfaces/display-vp-map.md`).
//! The display task decodes them, picks the screen and draws it on the panel over UART.

use companion_ui::dgus::Link;
use companion_ui::director::{Director, Settings};
use companion_ui::model::Model;
use companion_ui::ui::Ui;
use companion_ui::vp::Inputs;
use companion_ui::vp_map;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};
use esp_hal::{Async, uart::UartTx};
use esp_println::println;
use static_cell::ConstStaticCell;

/// Sent for a reading that's missing or faulted; the tile shows "Waiting".
pub const NO_READING: u16 = 0x8000;

/// How often the panel is redrawn. Only changed words are sent, so most frames are a few bytes.
const FRAME_MS: u64 = 50;

/// Latest sensor block from the main loop. A newer one replaces one the display hasn't taken yet.
static SENSOR_WORDS: Signal<CriticalSectionRawMutex, [u16; 10]> = Signal::new();
/// Latest clock from SNTP: Unix time (high word, low word) and the UTC offset in minutes.
static CLOCK_WORDS: Signal<CriticalSectionRawMutex, [u16; 3]> = Signal::new();

// Too big for a task's stack: the UI keeps a copy of the panel's drawing memory
static UI: ConstStaticCell<Ui> = ConstStaticCell::new(Ui::new());
static INPUTS: ConstStaticCell<Inputs> = ConstStaticCell::new(Inputs::new());

/// One snapshot of this node's readings, in physical units. `None` means no reading.
#[derive(Debug, Default, Clone, Copy)]
pub struct Readings {
    pub temperature_c: Option<f32>,
    pub humidity_percent: Option<f32>,
    pub voc_index: Option<i32>,
    pub co2_ppm: Option<u16>,
    pub lux: Option<f32>,
    pub distance_mm: Option<u16>,
    pub pm1_0: Option<u16>,
    pub pm2_5: Option<u16>,
    pub pm10: Option<u16>,
    pub presence: Option<bool>,
}

impl Readings {
    /// Encodes the readings as the 10 words at `vp_map::TEMPERATURE`, in map order.
    pub fn to_words(&self) -> [u16; 10] {
        [
            // °C × 10, signed
            self.temperature_c.map_or(NO_READING, |t| round(t * 10.0) as i16 as u16),
            // %RH × 10
            self.humidity_percent.map_or(NO_READING, |h| round(h.clamp(0.0, 100.0) * 10.0) as u16),
            // The gas index algorithm reports 0 while it warms up, so only 1–500 counts as a reading
            self.voc_index
                .filter(|v| (1..=500).contains(v))
                .map_or(NO_READING, |v| v as u16),
            self.co2_ppm.map_or(NO_READING, |c| c.min(0x7FFF)),
            // Whole lux, clamped to 32767 so it can't collide with NO_READING
            self.lux.map_or(NO_READING, |l| round(l.clamp(0.0, 32767.0)) as u16),
            self.distance_mm.map_or(NO_READING, |d| d.min(0x7FFF)),
            self.pm1_0.map_or(NO_READING, |p| p.min(0x7FFF)),
            self.pm2_5.map_or(NO_READING, |p| p.min(0x7FFF)),
            self.pm10.map_or(NO_READING, |p| p.min(0x7FFF)),
            self.presence.map_or(NO_READING, u16::from),
        ]
    }
}

/// Rounds to the nearest whole number (core has no `f32::round` without std).
/// The `as` casts that follow saturate, so out-of-range values can't wrap.
fn round(x: f32) -> f32 {
    if x >= 0.0 { x + 0.5 } else { x - 0.5 }
}

/// Hands the latest readings to the display task. Call once per sensor pass.
pub fn publish(readings: &Readings) {
    SENSOR_WORDS.signal(readings.to_words());
}

/// Sets the display clock. The display keeps counting from here, so call it once per sync.
/// `utc_offset_min` is local time minus UTC, e.g. -300 for EST.
pub fn set_clock(unix_seconds: u32, utc_offset_min: i16) {
    CLOCK_WORDS.signal([(unix_seconds >> 16) as u16, unix_seconds as u16, utc_offset_min as u16]);
}

/// Draws the screens on the panel. `tx` is a UART at 115200 8N1 wired to the panel's RX2.
#[embassy_executor::task]
pub async fn task(tx: UartTx<'static, Async>) {
    let mut link = Link::new(tx);
    let ui = UI.take();
    let inputs = INPUTS.take();
    let mut model = Model::default();
    let mut director = Director::new(Settings::default());

    loop {
        let now = Instant::now().as_millis();
        if let Some(words) = SENSOR_WORDS.try_take()
            && let Err(err) = inputs.write(vp_map::TEMPERATURE, &words, now)
        {
            println!("Display refused the sensor block: {:?}", err);
        }
        // unix_time (2 words) and utc_offset are consecutive, so one write covers all three
        if let Some(words) = CLOCK_WORDS.try_take()
            && let Err(err) = inputs.write(vp_map::UNIX_TIME, &words, now)
        {
            println!("Display refused the clock: {:?}", err);
        }

        inputs.apply(&mut model, &mut director.settings, now);
        // decide() also tracks presence and boot timing, so it runs every frame
        let decided = director.decide(&model, now).screen;
        let screen = inputs.screen().unwrap_or(decided);

        if let Err(err) = ui.frame(&mut link, screen, &model, now).await {
            println!("Display UART error: {:?}", err);
            // Unknown what reached the panel, so redraw everything next frame
            ui.invalidate();
        }
        Timer::after(Duration::from_millis(FRAME_MS)).await;
    }
}
