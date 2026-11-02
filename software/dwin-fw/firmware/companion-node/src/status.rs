//! waveshare status panel

use core::cell::Cell;
use core::fmt::Write;

use companion_ui::lightbar::{self, Pattern, Rgb};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::mono_font::ascii::{FONT_6X10, FONT_7X13_BOLD, FONT_10X20};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, CornerRadii, PrimitiveStyle, Rectangle, RoundedRectangle,
};
use embedded_graphics::text::{Baseline, Text};
use embedded_hal_bus::spi::{ExclusiveDevice, NoDelay};
use esp_hal::Blocking;
use esp_hal::gpio::Output;
use esp_hal::spi::master::Spi;
use heapless::String;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;

use crate::ledbar::LEDS;

pub type Lcd = mipidsi::Display<
    SpiInterface<
        'static,
        ExclusiveDevice<Spi<'static, Blocking>, Output<'static>, NoDelay>,
        Output<'static>,
    >,
    ST7789,
    Output<'static>,
>;

/// dwin link state
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Link {
    Searching,
    Factory,
    Ours,
}

#[derive(Clone, Copy)]
pub struct Status {
    pub link: Link,
    pub scene: &'static str,
    pub scene_index: u8,
    pub scene_count: u8,
    pub scene_ms: u32,
    pub scene_len_ms: u32,
    pub page: u16,
    pub bytes: u32,
    pub version: u16,
    pub rx_bytes: u32,
    pub rx_errors: u32,
    pub baud: u32,
    pub pattern: Pattern,
    pub pattern_start: u64,
    /// live mode flag
    pub live: bool,
    /// write count statistics
    pub writes: u32,
    pub rejected: u32,
}

pub static STATUS: Mutex<CriticalSectionRawMutex, Cell<Status>> = Mutex::new(Cell::new(Status {
    link: Link::Searching,
    scene: "",
    scene_index: 0,
    scene_count: 0,
    scene_ms: 0,
    scene_len_ms: 1,
    page: 0,
    bytes: 0,
    version: 0,
    rx_bytes: 0,
    rx_errors: 0,
    baud: 115_200,
    pattern: Pattern::Off,
    pattern_start: 0,
    live: false,
    writes: 0,
    rejected: 0,
}));

const fn rgb(r: u8, g: u8, b: u8) -> Rgb565 {
    Rgb565::new(r >> 3, g >> 2, b >> 3)
}

const BG: Rgb565 = rgb(11, 15, 20);
const SURFACE: Rgb565 = rgb(31, 39, 50);
const CHANNEL: Rgb565 = rgb(7, 9, 11);
const TEXT: Rgb565 = rgb(234, 240, 246);
const MUTED: Rgb565 = rgb(138, 151, 168);
const ACCENT: Rgb565 = rgb(94, 234, 212);
const WARN: Rgb565 = rgb(251, 191, 36);
const BAD: Rgb565 = rgb(239, 68, 68);
const LED_OFF: Rgb = Rgb::new(21, 25, 30);

const W: u32 = 320;
const LED_W: u32 = 10;
const LED_GAP: u32 = 2;
const LED_X0: i32 = ((W - (LEDS as u32 * (LED_W + LED_GAP) - LED_GAP)) / 2) as i32;
const LED_Y: i32 = 103;

fn pattern_name(p: &Pattern) -> &'static str {
    match p {
        Pattern::Off => "off",
        Pattern::Calm(_) => "calm (air colour)",
        Pattern::Meter(_) => "level meter",
        Pattern::Thinking => "thinking",
        Pattern::Unread => "unread statement",
        Pattern::Confirm => "action confirmed",
        Pattern::Alert => "alert",
        Pattern::Offline => "hub offline",
        Pattern::Night => "night",
        Pattern::Greeting => "greeting",
        Pattern::Boot { .. } => "boot",
        Pattern::Setup => "setup",
        Pattern::Rainbow => "rainbow",
        Pattern::Test => "test: red green blue white",
        Pattern::Rooms { .. } => "room segments",
    }
}

/// diffuse pixel color
fn led_colour(c: Rgb) -> Rgb565 {
    let a = c.r.max(c.g).max(c.b) as f32 / 255.0;
    if a < 0.01 {
        return rgb(LED_OFF.r, LED_OFF.g, LED_OFF.b);
    }
    let hue = |v: u8| (v as f32 / a).min(255.0);
    let k = (a * 1.6).min(1.0);
    let mix = |off: u8, v: u8| (off as f32 + (hue(v) - off as f32) * k) as u8;
    rgb(
        mix(LED_OFF.r, c.r),
        mix(LED_OFF.g, c.g),
        mix(LED_OFF.b, c.b),
    )
}

fn text(lcd: &mut Lcd, s: &str, x: i32, y: i32, style: MonoTextStyle<'_, Rgb565>) {
    let _ = Text::with_baseline(s, Point::new(x, y), style, Baseline::Top).draw(lcd);
}

fn clear(lcd: &mut Lcd, x: i32, y: i32, w: u32, h: u32) {
    let _ = lcd.fill_solid(&Rectangle::new(Point::new(x, y), Size::new(w, h)), BG);
}

#[embassy_executor::task]
pub async fn task(mut lcd: Lcd, _backlight: Output<'static>, _sd_cs: Output<'static>) {
    let _ = lcd.clear(BG);
    text(
        &mut lcd,
        "IntelliThings",
        10,
        8,
        MonoTextStyle::new(&FONT_7X13_BOLD, ACCENT),
    );
    text(
        &mut lcd,
        "companion bench",
        10,
        22,
        MonoTextStyle::new(&FONT_6X10, MUTED),
    );
    let _ = RoundedRectangle::new(
        Rectangle::new(
            Point::new(LED_X0 - 8, LED_Y - 7),
            Size::new(W - 2 * (LED_X0 as u32 - 8), 28),
        ),
        CornerRadii::new(Size::new(14, 14)),
    )
    .into_styled(PrimitiveStyle::with_fill(CHANNEL))
    .draw(&mut lcd);

    let mut last: Option<Status> = None;
    let mut leds = [Rgb565::BLACK; LEDS];
    let mut shown_leds = [Rgb565::new(1, 2, 3); LEDS];
    let mut frame = [Rgb::default(); LEDS];
    let mut last_stats = 0u64;
    loop {
        let s = STATUS.lock(|c| c.get());
        let now = Instant::now().as_millis();

        // draw link chip
        if last.is_none_or(|l| l.link != s.link) {
            clear(&mut lcd, 200, 6, 120, 14);
            let (colour, label) = match s.link {
                Link::Searching => (WARN, "SEARCHING"),
                Link::Factory => (WARN, "FACTORY UI"),
                Link::Ours => (ACCENT, "PANEL OK"),
            };
            let _ = Circle::new(Point::new(310 - 6 * label.len() as i32 - 14, 9), 8)
                .into_styled(PrimitiveStyle::with_fill(colour))
                .draw(&mut lcd);
            text(
                &mut lcd,
                label,
                310 - 6 * label.len() as i32,
                8,
                MonoTextStyle::new(&FONT_6X10, colour),
            );
        }

        // draw scene name
        if last.is_none_or(|l| l.scene != s.scene || l.page != s.page || l.live != s.live) {
            clear(&mut lcd, 0, 38, W, 34);
            text(
                &mut lcd,
                s.scene,
                10,
                38,
                MonoTextStyle::new(&FONT_10X20, TEXT),
            );
            let mut line: String<40> = String::new();
            let _ = if s.live {
                write!(line, "live data  -  DWIN page {}", s.page)
            } else {
                write!(
                    line,
                    "demo {}/{}  -  DWIN page {}",
                    s.scene_index + 1,
                    s.scene_count,
                    s.page
                )
            };
            text(
                &mut lcd,
                &line,
                10,
                61,
                MonoTextStyle::new(&FONT_6X10, MUTED),
            );
        }

        // draw scene progress
        let done =
            (s.scene_ms.min(s.scene_len_ms) as u64 * 300 / s.scene_len_ms.max(1) as u64) as u32;
        let _ = lcd.fill_solid(
            &Rectangle::new(Point::new(10, 78), Size::new(done, 3)),
            ACCENT,
        );
        let _ = lcd.fill_solid(
            &Rectangle::new(Point::new(10 + done as i32, 78), Size::new(300 - done, 3)),
            SURFACE,
        );

        // draw simulated lightbar
        let t = now.saturating_sub(s.pattern_start) as f32 / 1000.0;
        lightbar::render(&s.pattern, t, 1.0, &mut frame);
        for (i, c) in frame.iter().enumerate() {
            leds[i] = led_colour(*c);
        }
        for i in 0..LEDS {
            if leds[i] != shown_leds[i] {
                let x = LED_X0 + i as i32 * (LED_W + LED_GAP) as i32;
                let _ = lcd.fill_solid(
                    &Rectangle::new(Point::new(x, LED_Y), Size::new(LED_W, 14)),
                    leds[i],
                );
                shown_leds[i] = leds[i];
            }
        }
        if last.is_none_or(|l| {
            core::mem::discriminant(&l.pattern) != core::mem::discriminant(&s.pattern)
        }) {
            clear(&mut lcd, 0, 130, W, 12);
            let mut line: String<40> = String::new();
            let _ = write!(line, "light bar: {}", pattern_name(&s.pattern));
            text(
                &mut lcd,
                &line,
                10,
                131,
                MonoTextStyle::new(&FONT_6X10, MUTED),
            );
        }

        // draw traffic statistics
        if now - last_stats > 1000 {
            last_stats = now;
            clear(&mut lcd, 0, 150, W, 12);
            let mut line: String<48> = String::new();
            match s.link {
                Link::Searching => {
                    let _ = write!(
                        line,
                        "{} baud: rx {} B, {} err",
                        s.baud, s.rx_bytes, s.rx_errors
                    );
                }
                _ if s.live || s.writes > 0 => {
                    let _ = write!(
                        line,
                        "VP writes {} ({} refused)  sent {}.{} kB",
                        s.writes,
                        s.rejected,
                        s.bytes / 1000,
                        s.bytes % 1000 / 100
                    );
                }
                _ => {
                    let _ = write!(
                        line,
                        "sent {}.{} kB  DGUS v{:02X} OS v{:02X}",
                        s.bytes / 1000,
                        s.bytes % 1000 / 100,
                        s.version >> 8,
                        s.version & 0xFF
                    );
                }
            }
            let colour = if s.link == Link::Searching {
                BAD
            } else {
                MUTED
            };
            text(
                &mut lcd,
                &line,
                10,
                151,
                MonoTextStyle::new(&FONT_6X10, colour),
            );
        }

        last = Some(s);
        Timer::after(Duration::from_millis(66)).await;
    }
}
