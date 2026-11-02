//! screen rendering engine

use core::f32::consts::PI;
use core::fmt::Write;

use heapless::String;
use libm::{cosf, fabsf, sinf};

use crate::air::{self, Judgement, State};
use crate::canvas::{Canvas, Layer};
use crate::clock;
use crate::dgus::VpImage;
use crate::generated::{self, sprite, style};
use crate::model::{Chip, Device, Hub, Metric, Model, Readings, Step};
use crate::text::{Align, GlyphStyle};

/// display screen variant
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Boot,
    Home,
    Sensors,
    Rooms,
    Alert,
    Night,
    Away,
    Setup,
    Test,
}

impl Screen {
    pub fn name(self) -> &'static str {
        match self {
            Screen::Boot => "boot",
            Screen::Home => "home",
            Screen::Sensors => "sensors",
            Screen::Rooms => "rooms",
            Screen::Alert => "alert",
            Screen::Night => "night",
            Screen::Away => "away",
            Screen::Setup => "setup",
            Screen::Test => "test pattern",
        }
    }

    pub fn page(self) -> u16 {
        match self {
            Screen::Boot => generated::page::BOOT,
            Screen::Home => generated::page::HOME,
            Screen::Sensors => generated::page::SENSORS,
            Screen::Rooms => generated::page::ROOMS,
            Screen::Alert => generated::page::ALERT,
            Screen::Night => generated::page::NIGHT,
            Screen::Away => generated::page::AWAY,
            Screen::Setup => generated::page::SETUP,
            Screen::Test => generated::page::TEST,
        }
    }
}

/// extra frame elements
pub struct Extras<'a, const N: usize> {
    pub img: &'a mut VpImage<N>,
}

const CPS: u64 = 38; // chars per second
/// pulse alert border
const ALERT_BORDER_PULSE: bool = false;

// screen helper functions

fn si(s: State) -> u16 {
    s as u16
}

/// alert state index
fn ai(s: State) -> u16 {
    match s {
        State::Moderate => 0,
        State::Poor => 1,
        _ => 2,
    }
}

fn frame(now_ms: u64, period_ms: u64, frames: u64) -> u16 {
    ((now_ms % period_ms) * frames / period_ms) as u16
}

fn grouped(n: u32) -> String<12> {
    let mut s: String<12> = String::new();
    if n >= 1000 {
        let _ = write!(s, "{},{:03}", n / 1000, n % 1000);
    } else {
        let _ = write!(s, "{}", n);
    }
    s
}

/// display reading tuple
pub struct Shown {
    pub value: String<12>,
    pub unit: &'static str,
    pub j: Judgement,
}

pub fn shown(m: Metric, r: &Readings, fahrenheit: bool) -> Shown {
    let mut value: String<12> = String::new();
    let (unit, j) = match m {
        Metric::Temp => match r.temp_c {
            Some(c) => {
                let v = if fahrenheit { c * 1.8 + 32.0 } else { c };
                let _ = write!(value, "{:.1}", v);
                (if fahrenheit { "°F" } else { "°C" }, air::temperature(c))
            }
            None => ("", air::waiting()),
        },
        Metric::Humidity => match r.humidity {
            Some(h) => {
                let _ = write!(value, "{:.0}", h);
                ("%", air::humidity(h))
            }
            None => ("", air::waiting()),
        },
        Metric::Co2 => match r.co2 {
            Some(p) => {
                value = grouped(p as u32);
                ("ppm", air::co2(p))
            }
            None => ("", air::waiting()),
        },
        Metric::Pm2_5 => match r.pm2_5 {
            Some(p) => {
                let _ = write!(value, "{}", p);
                ("µg/m³", air::pm2_5(p))
            }
            None => ("", air::waiting()),
        },
        Metric::Voc => match r.voc {
            Some(v) => {
                let _ = write!(value, "{}", v);
                ("", air::voc_index(v))
            }
            None => ("", air::waiting()),
        },
        Metric::Light => match r.lux {
            Some(l) => {
                value = grouped(l.max(0.0) as u32);
                ("lx", air::light(l))
            }
            None => ("", air::waiting()),
        },
    };
    if value.is_empty() {
        let _ = value.push_str("–");
    }
    Shown { value, unit, j }
}

/// worst air state
pub fn overall(r: &Readings) -> State {
    let mut s = State::Neutral;
    for m in [
        Metric::Temp,
        Metric::Humidity,
        Metric::Co2,
        Metric::Pm2_5,
        Metric::Voc,
    ] {
        let j = shown(m, r, false).j;
        if j.word != "Waiting" {
            s = s.worst(j.state);
        }
    }
    if s == State::Neutral { State::Good } else { s }
}

fn pick(
    s: State,
    good: &'static GlyphStyle,
    moderate: &'static GlyphStyle,
    poor: &'static GlyphStyle,
    bad: &'static GlyphStyle,
    neutral: &'static GlyphStyle,
) -> &'static GlyphStyle {
    match s {
        State::Good => good,
        State::Moderate => moderate,
        State::Poor => poor,
        State::Bad => bad,
        State::Neutral => neutral,
    }
}

fn room_state(s: State) -> State {
    if s == State::Neutral { State::Good } else { s }
}

// shared ui components

#[derive(Clone, Copy, PartialEq, Eq)]
enum Bar {
    Normal,
    Alert,
}

/// render status bar
fn top_bar(
    c: &mut Canvas,
    m: &Model,
    now: u64,
    title: &str,
    label: Option<&str>,
    baked: Option<&str>,
    bar: Bar,
) {
    let (title_style, label_style) = match bar {
        Bar::Normal => (&style::TITLE, &style::TOPBAR),
        Bar::Alert => (&style::ALERT_TITLE_BAR, &style::ALERT_TOPBAR),
    };
    c.text(Layer::Main, title_style, title, 24, 33, Align::Left);
    let mut x: i16 = 776;
    let mut synced: String<48> = String::new();
    let offline = m.hub == Hub::Offline && bar == Bar::Normal;
    if let Some(b) = baked {
        x -= label_style.width(b) + 14;
    } else {
        let text = match label {
            Some(t) => t,
            None => match (m.hub, m.synced_ms) {
                (Hub::Offline, _) => "Hub offline",
                (Hub::Connecting, _) => "Connecting…",
                (Hub::Online, Some(at)) => {
                    let age = clock::age(((now.saturating_sub(at)) / 1000) as i64);
                    let _ = write!(synced, "Synced {}", age.as_str());
                    synced.as_str()
                }
                (Hub::Online, None) => "Online",
            },
        };
        let st = if offline {
            &style::TOPBAR_WARN
        } else {
            label_style
        };
        x -= c.text(Layer::Main, st, text, x, 33, Align::Right) + 14;
    }
    let (hub, wifi, person) = match bar {
        Bar::Normal => (
            if offline {
                sprite::HUB_OFF
            } else {
                sprite::HUB
            },
            if m.wifi {
                sprite::WIFI
            } else {
                sprite::WIFI_OFF
            },
            sprite::PERSON,
        ),
        Bar::Alert => (sprite::ALERT_HUB, sprite::ALERT_WIFI, sprite::ALERT_PERSON),
    };
    c.sprite_at(Layer::Main, hub, x - 8, 25);
    if offline {
        c.sprite_at(Layer::Main, sprite::HUB_OFF_DOT, x - 1, 18);
    }
    x -= 32;
    c.sprite_at(Layer::Main, wifi, x - 8, 23);
    x -= 32;
    if m.here.presence {
        c.sprite_at(Layer::Main, person, x - 8, 24);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ChipWhere {
    Card = 0,
    Alert = 1,
}

/// render action chip
#[allow(clippy::too_many_arguments)]
fn chip(
    c: &mut Canvas,
    layer: Layer,
    at: ChipWhere,
    ask: bool,
    ch: &Chip,
    x: i16,
    y: i16,
    done: bool,
) -> i16 {
    // match card glyphs
    let (a_style, b_style) = if ask {
        (&style::CHIP_CARD_ASK, &style::CHIP_CARD_ASK_HI)
    } else {
        (&style::CHIP_CARD_ACT, &style::CHIP_CARD_ACT_HI)
    };
    let has_b = !ch.value.is_empty();
    let w = 16
        + 22
        + a_style.width(&ch.label)
        + if has_b {
            26 + b_style.width(&ch.value)
        } else {
            0
        }
        + 16
        + if done { 22 } else { 0 };
    // chip sprite block
    let base = sprite::CHIP_CARD_ACT_L + 13 * (2 * at as u16 + ask as u16);
    const CAP: i16 = 18;
    const SLICE: i16 = 16;
    c.sprite(Layer::Skin, base, x, y);
    let mut mx = x + CAP;
    while mx < x + w - CAP {
        let sx = mx.min(x + w - CAP - SLICE);
        c.sprite(Layer::Skin, base + 1, sx, y);
        mx += SLICE;
    }
    c.sprite(Layer::Skin, base + 2, x + w - CAP, y);
    c.sprite_at(layer, base + 3 + ch.device as u16, x + 26, y + 17);
    let mut cx = x + 40;
    cx += c.text(layer, a_style, &ch.label, cx, y + 23, Align::Left);
    if has_b {
        c.sprite_at(layer, base + 11, cx + 13, y + 17);
        cx += 26;
        cx += c.text(layer, b_style, &ch.value, cx, y + 23, Align::Left);
    }
    if done {
        c.sprite_at(layer, base + 12, cx + 14, y + 17);
    }
    w
}

// home screen renderer

pub fn tile_origin(i: usize) -> (i16, i16) {
    (412 + (i as i16 % 2) * 188, 60 + (i as i16 / 2) * 136)
}

const CARD: (i16, i16, i16, i16) = (24, 232, 364, 224);
const IDLE_TEXT: &str = "No updates yet. I'll post here when something in the room changes.";
const OFFLINE_TEXT: &str = "I can't reach the home hub, so there are no AI updates for now. This room's readings are still live.";

fn home<const N: usize>(c: &mut Canvas, x: &mut Extras<N>, m: &Model, now: u64) {
    top_bar(c, m, now, &m.room, None, None, Bar::Normal);
    // render clock digits
    match m.time(now) {
        Some(t) => {
            let hm = t.hhmm(m.clock_24h);
            let w = c.text(Layer::Main, &style::CLOCK, &hm, 24, 150, Align::Left);
            if !m.clock_24h {
                c.text(
                    Layer::Main,
                    &style::AMPM,
                    t.ampm(),
                    24 + w + 8,
                    150,
                    Align::Left,
                );
            }
            c.text(
                Layer::Main,
                &style::DATE,
                &t.date_line(),
                28,
                192,
                Align::Left,
            );
        }
        None => {
            c.text(Layer::Main, &style::CLOCK, "--:--", 24, 150, Align::Left);
        }
    }
    // render metric tiles
    for (i, metric) in Metric::ALL.iter().enumerate() {
        let (tx, ty) = tile_origin(i);
        let s = shown(*metric, &m.here, m.fahrenheit);
        let st = s.j.state;
        if st.is_alert() {
            c.sprite(
                Layer::Skin,
                sprite::TILE_TEMP_MODERATE + 3 * i as u16 + ai(st),
                tx,
                ty,
            );
        }
        let vs = pick(
            st,
            &style::TILE_VALUE_GOOD,
            &style::TILE_VALUE_MODERATE,
            &style::TILE_VALUE_POOR,
            &style::TILE_VALUE_BAD,
            &style::TILE_VALUE_NEUTRAL,
        );
        let us = pick(
            st,
            &style::TILE_UNIT_GOOD,
            &style::TILE_UNIT_MODERATE,
            &style::TILE_UNIT_POOR,
            &style::TILE_UNIT_BAD,
            &style::TILE_UNIT_NEUTRAL,
        );
        let ws = pick(
            st,
            &style::TILE_WORD_GOOD,
            &style::TILE_WORD_MODERATE,
            &style::TILE_WORD_POOR,
            &style::TILE_WORD_BAD,
            &style::TILE_WORD_NEUTRAL,
        );
        let vw = c.text(Layer::Main, vs, &s.value, tx + 18, ty + 84, Align::Left);
        c.text(
            Layer::Main,
            us,
            s.unit,
            tx + 18 + vw + 5,
            ty + 84,
            Align::Left,
        );
        c.sprite_at(
            Layer::Main,
            sprite::STATE_DOT_GOOD + si(st),
            tx + 23,
            ty + 124 - 21,
        );
        c.text(
            Layer::Main,
            ws,
            s.j.word,
            tx + 34,
            ty + 124 - 15,
            Align::Left,
        );
    }
    agent_card(c, m, now);
    toast(c, x, m, now);
}

fn agent_card(c: &mut Canvas, m: &Model, now: u64) {
    let (x, y, w, h) = CARD;
    let offline = m.hub == Hub::Offline;
    let thinking = m.thinking.is_some() && !offline;
    let orb = if offline {
        sprite::ORB_OFFLINE_0
    } else if thinking {
        sprite::ORB_THINKING_0 + frame(now, 2400, 16)
    } else {
        sprite::ORB_IDLE_0 + frame(now, 3200, 16)
    };
    c.sprite_at(Layer::Main, orb, x + 30, y + 32);
    let (header, hstyle) = if offline {
        ("Hub offline", &style::CARD_HEADER_WARN)
    } else {
        ("IntelliThings", &style::CARD_HEADER)
    };
    c.text(Layer::Main, hstyle, header, x + 52, y + 38, Align::Left);

    let mut age: String<24> = String::new();
    let mut reveal: Option<usize> = None;
    let mut chips: &[Chip] = &[];
    let text: &str = if offline {
        if let Some(at) = m.retry_at_ms {
            let s = at.saturating_sub(now) / 1000;
            let _ = write!(age, "retry in {}:{:02}", s / 60, s % 60);
        }
        OFFLINE_TEXT
    } else if let Some(t) = &m.thinking {
        t
    } else if let Some(st) = &m.statement {
        let secs = match (st.ts, m.unix_now(now)) {
            (Some(ts), Some(n)) => n - ts,
            _ => ((now.saturating_sub(st.at_ms)) / 1000) as i64,
        };
        let _ = age.push_str(&clock::age(secs.max(0)));
        let n = ((now.saturating_sub(st.at_ms)) * CPS / 1000) as usize;
        let total = st.text.chars().count();
        if n < total {
            reveal = Some(n);
        } else {
            chips = &st.chips;
        }
        &st.text
    } else {
        IDLE_TEXT
    };
    if !age.is_empty() {
        c.text(
            Layer::Main,
            &style::CARD_AGE,
            &age,
            x + w - 20,
            y + 38,
            Align::Right,
        );
    }
    let max_lines = if chips.is_empty() { 4 } else { 3 };
    let mut left = reveal.unwrap_or(usize::MAX);
    let mut caret: Option<(i16, i16)> = None;
    style::STATEMENT.wrap(text, w - 40, max_lines, |i, line, truncated| {
        let ly = y + 84 + i as i16 * 30;
        let count = line.chars().count();
        let shown_n = count.min(left);
        let end = line
            .char_indices()
            .nth(shown_n)
            .map_or(line.len(), |(b, _)| b);
        let lw = c.text(
            Layer::Main,
            &style::STATEMENT,
            &line[..end],
            x + 20,
            ly,
            Align::Left,
        );
        if truncated && shown_n == count {
            c.text(
                Layer::Main,
                &style::STATEMENT,
                "…",
                x + 20 + lw,
                ly,
                Align::Left,
            );
        }
        if reveal.is_some() && left <= count && caret.is_none() {
            caret = Some((x + 20 + lw + 2, y + 64 + i as i16 * 30));
        }
        left = left.saturating_sub(count + 1);
    });
    if let Some((cx, cy)) = caret
        && (now / 250).is_multiple_of(2)
    {
        c.sprite(Layer::Main, sprite::CARET, cx, cy);
    }
    let mut cx = x + 20;
    for ch in chips {
        cx += chip(
            c,
            Layer::Main,
            ChipWhere::Card,
            false,
            ch,
            cx,
            y + h - 54,
            true,
        ) + 8;
    }
}

const TOAST_MS: u64 = 5500;

fn toast<const N: usize>(c: &mut Canvas, x: &mut Extras<N>, m: &Model, now: u64) {
    let mut scrim = 100u16;
    if let Some(t) = &m.toast {
        let age = now.saturating_sub(t.at_ms);
        if age < TOAST_MS {
            // scrim fade window
            let edge = age.min(TOAST_MS - age).min(300);
            scrim = 100 - (35 * edge / 300) as u16;
            let y = 14;
            c.sprite(Layer::Over, sprite::TOAST_CARD, 140, y);
            c.sprite_at(
                Layer::Over,
                sprite::TOAST_FAN + t.device as u16,
                184,
                y + 39,
            );
            c.text(
                Layer::Over,
                &style::TOAST_TITLE,
                &t.title,
                220,
                y + 34,
                Align::Left,
            );
            c.text(
                Layer::Over,
                &style::TOAST_SUB,
                &t.detail,
                220,
                y + 58,
                Align::Left,
            );
        }
    }
    x.img.set(generated::SCRIM_VP, scrim);
}

// sensors screen renderer

fn sensors<const N: usize>(c: &mut Canvas, x: &mut Extras<N>, m: &Model, now: u64) {
    let mut title: String<48> = String::new();
    let _ = write!(title, "{} · Sensors", m.room.as_str());
    top_bar(c, m, now, &title, None, Some("Last 12 hours"), Bar::Normal);
    for (i, metric) in Metric::ALL.iter().enumerate() {
        let y = 60 + 67 * i as i16;
        let s = shown(*metric, &m.here, m.fahrenheit);
        let st = s.j.state;
        if st.is_alert() {
            c.sprite_at(
                Layer::Main,
                sprite::ROW_TEMP_MODERATE + 3 * i as u16 + ai(st),
                52,
                y + 30,
            );
        }
        c.text(
            Layer::Main,
            &style::ROW_VALUE,
            &s.value,
            282,
            y + 39,
            Align::Right,
        );
        c.text(
            Layer::Main,
            &style::ROW_UNIT,
            s.unit,
            288,
            y + 39,
            Align::Left,
        );
        let colour = trend_colour(st);
        let last = trend_line(x, i, &m.trends.values[i][..m.trends.len], y, colour);
        if let Some((px, py)) = last {
            c.sprite_at(
                Layer::Main,
                sprite::PULSE_GOOD_0 + 8 * si(st) + frame(now, 1430, 8),
                px,
                py,
            );
        }
        c.sprite_at(Layer::Main, sprite::ROW_DOT_GOOD + si(st), 646, y + 30);
        let ws = pick(
            st,
            &style::ROW_WORD_GOOD,
            &style::ROW_WORD_MODERATE,
            &style::ROW_WORD_POOR,
            &style::ROW_WORD_BAD,
            &style::ROW_WORD_NEUTRAL,
        );
        c.text(Layer::Main, ws, s.j.word, 760, y + 35, Align::Right);
    }
}

fn rgb565(r: u8, g: u8, b: u8) -> u16 {
    ((r as u16 >> 3) << 11) | ((g as u16 >> 2) << 5) | (b as u16 >> 3)
}

fn trend_colour(s: State) -> u16 {
    match s {
        State::Good => rgb565(74, 222, 128),
        State::Moderate => rgb565(251, 191, 36),
        State::Poor => rgb565(249, 115, 22),
        State::Bad => rgb565(239, 68, 68),
        State::Neutral => rgb565(138, 151, 168),
    }
}

/// write trend polylines
fn trend_line<const N: usize>(
    x: &mut Extras<N>,
    i: usize,
    data: &[f32],
    y: i16,
    colour: u16,
) -> Option<(i16, i16)> {
    let (x0, x1, top, bot) = (360i16, 612i16, y + 12, y + 47);
    let mut last = None;
    for k in 0..2 {
        let vp = generated::TREND_VP[2 * i + k];
        if data.len() < 2 {
            x.img.set_slice(vp, &[0x0002, 0, colour, 0xFF00]);
            continue;
        }
        let (lo, hi) = data
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
        let span = if hi - lo > 1e-6 { hi - lo } else { 1.0 };
        x.img
            .set_slice(vp, &[0x0002, (data.len() - 1) as u16, colour]);
        for (n, &v) in data.iter().enumerate() {
            let px = x0 + ((x1 - x0) as f32 * n as f32 / (data.len() - 1) as f32) as i16;
            let py = bot - ((v - lo) / span * (bot - top) as f32) as i16;
            x.img
                .set_slice(vp + 3 + 2 * n as u16, &[px as u16, (py + k as i16) as u16]);
            last = Some((px, py));
        }
        x.img.set(vp + 3 + 2 * data.len() as u16, 0xFF00);
    }
    last
}

// rooms screen renderer

fn rooms(c: &mut Canvas, m: &Model, now: u64) {
    let nodes = generated::ROOMS.iter().filter(|r| r.4).count();
    let online = m
        .rooms
        .iter()
        .zip(generated::ROOMS.iter())
        .filter(|(s, g)| g.4 && s.online)
        .count();
    let mut right: String<48> = String::new();
    match (m.hub, m.synced_ms) {
        (Hub::Online, Some(at)) => {
            let _ = write!(
                right,
                "{} of {} sensor nodes · synced {}",
                online,
                nodes,
                clock::age(((now.saturating_sub(at)) / 1000) as i64).as_str()
            );
        }
        _ => {
            let _ = write!(right, "{} of {} sensor nodes", online, nodes);
        }
    }
    top_bar(c, m, now, "Home", Some(&right), None, Bar::Normal);
    let mut node = 0u16;
    for (i, (geo, r)) in generated::ROOMS.iter().zip(m.rooms.iter()).enumerate() {
        let (x, y, w, h, has_node) = *geo;
        if !has_node {
            if !r.note.is_empty() {
                c.sprite_at(Layer::Main, sprite::ROOM_BULB, x + w - 30, y + 26);
                let st: &GlyphStyle = &style::ROOM_NOTE;
                c.text(Layer::Main, st, &r.note, x + w - 44, y + 32, Align::Right);
            }
            continue;
        }
        let here = i == m.room_index;
        let st = room_state(r.state.unwrap_or(State::Good));
        if st != State::Good || here {
            let off = if st == State::Good {
                0
            } else {
                1 + 2 * (si(st) - 1) + here as u16
            };
            c.sprite(Layer::Skin, sprite::ROOM0_GOOD_HERE + 7 * node + off, x, y);
        }
        node += 1;
        let big = h > 200;
        let ty = if big { y + h - 52 } else { y + h - 40 };
        let ts = if big {
            pick(
                st,
                &style::ROOM_BIG_GOOD,
                &style::ROOM_BIG_MODERATE,
                &style::ROOM_BIG_POOR,
                &style::ROOM_BIG_BAD,
                &style::ROOM_BIG_GOOD,
            )
        } else {
            pick(
                st,
                &style::ROOM_SMALL_GOOD,
                &style::ROOM_SMALL_MODERATE,
                &style::ROOM_SMALL_POOR,
                &style::ROOM_SMALL_BAD,
                &style::ROOM_SMALL_GOOD,
            )
        };
        let hs = pick(
            st,
            &style::ROOM_HUM_GOOD,
            &style::ROOM_HUM_MODERATE,
            &style::ROOM_HUM_POOR,
            &style::ROOM_HUM_BAD,
            &style::ROOM_HUM_GOOD,
        );
        let ws = pick(
            st,
            &style::ROOM_WORD_GOOD,
            &style::ROOM_WORD_MODERATE,
            &style::ROOM_WORD_POOR,
            &style::ROOM_WORD_BAD,
            &style::ROOM_WORD_GOOD,
        );
        let mut t: String<8> = String::new();
        match (r.online, r.temp_c) {
            (true, Some(c_)) => {
                let v = if m.fahrenheit { c_ * 1.8 + 32.0 } else { c_ };
                let _ = write!(t, "{:.0}°", v);
            }
            _ => {
                let _ = t.push_str("--");
            }
        }
        let tw = c.text(Layer::Main, ts, &t, x + 18, ty, Align::Left);
        if let (true, Some(hum)) = (r.online, r.humidity) {
            let mut hs_: String<8> = String::new();
            let _ = write!(hs_, "{:.0}%", hum);
            c.text(Layer::Main, hs, &hs_, x + 26 + tw, ty, Align::Left);
        }
        if r.online && r.state.is_some() {
            c.sprite_at(
                Layer::Main,
                sprite::ROOM_DOT_GOOD + si(st),
                x + 23,
                y + h - 21,
            );
            c.text(Layer::Main, ws, r.word, x + 34, y + h - 15, Align::Left);
        }
        let ring = sprite::ROOM_RING_GOOD_0 + 8 * si(st);
        if r.online && r.presence {
            c.sprite_at(Layer::Main, ring + frame(now, 1250, 8), x + w - 30, y + 28);
        } else {
            c.sprite_at(
                Layer::Main,
                sprite::ROOM_PERSON_FAINT_GOOD + si(st),
                x + w - 30,
                y + 28,
            );
        }
    }
}

// alert screen renderer

fn alert(c: &mut Canvas, m: &Model, now: u64) {
    let Some(a) = &m.alert else {
        return;
    };
    let mut right: String<48> = String::new();
    let _ = write!(right, "Alert · {} node", a.place.as_str());
    top_bar(c, m, now, &m.room, Some(&right), None, Bar::Alert);
    // static border fallback
    if ALERT_BORDER_PULSE {
        let t = now as f32 / 1000.0;
        let k = (fabsf(sinf(t * PI)) * 5.0 + 0.5) as u16;
        let e = sprite::ALERT_EDGE_T_0 + 4 * k;
        c.sprite(Layer::Skin, e, 24, 60);
        c.sprite(Layer::Skin, e + 1, 24, 60 + 396 - 22);
        c.sprite(Layer::Skin, e + 2, 24, 60 + 22);
        c.sprite(Layer::Skin, e + 3, 24 + 752 - 4, 60 + 22);
    }

    c.text(
        Layer::Main,
        &style::ALERT_HEAD,
        &a.title,
        96,
        114,
        Align::Left,
    );
    let mut sub: String<64> = String::new();
    let _ = write!(
        sub,
        "{} · started {}",
        a.place.as_str(),
        clock::age(((now.saturating_sub(a.since_ms)) / 1000) as i64).as_str()
    );
    c.text(Layer::Main, &style::ALERT_SUB, &sub, 56, 150, Align::Left);
    let mut v: String<12> = String::new();
    let _ = write!(v, "{}", a.value);
    let vw = c.text(Layer::Main, &style::ALERT_VALUE, &v, 52, 270, Align::Left);
    let unit = match a.metric {
        Metric::Temp => {
            if m.fahrenheit {
                "°F"
            } else {
                "°C"
            }
        }
        Metric::Humidity => "%",
        Metric::Co2 => "ppm",
        Metric::Pm2_5 => "µg/m³",
        Metric::Voc => "index",
        Metric::Light => "lx",
    };
    c.text(
        Layer::Main,
        &style::ALERT_UNIT,
        unit,
        60 + vw,
        232,
        Align::Left,
    );
    c.text(
        Layer::Main,
        &style::ALERT_METRIC,
        a.metric.label(),
        60 + vw,
        262,
        Align::Left,
    );
    c.text(
        Layer::Main,
        &style::ALERT_SUB,
        &a.normal,
        56,
        306,
        Align::Left,
    );
    c.sprite_at(
        Layer::Main,
        // slow orb framerate
        sprite::ALERT_ORB_ALERT_0 + 2 * frame(now, 1600, 8),
        70,
        372,
    );
    style::ALERT_MSG.wrap(&a.message, 290, 3, |i, line, truncated| {
        let w = c.text(
            Layer::Main,
            &style::ALERT_MSG,
            line,
            92,
            378 + 26 * i as i16,
            Align::Left,
        );
        if truncated {
            c.text(
                Layer::Main,
                &style::ALERT_MSG,
                "…",
                92 + w,
                378 + 26 * i as i16,
                Align::Left,
            );
        }
    });
    for (k, ch) in a.done.iter().enumerate() {
        chip(
            c,
            Layer::Main,
            ChipWhere::Alert,
            false,
            ch,
            420,
            210 + 44 * k as i16,
            true,
        );
    }
    if let Some(ch) = &a.ask {
        chip(c, Layer::Main, ChipWhere::Alert, true, ch, 420, 338, false);
    }
}

// night away renderers

fn conditions_line(m: &Model, with_humidity: bool) -> String<48> {
    let mut s: String<48> = String::new();
    if let Some(c) = m.here.temp_c {
        let v = if m.fahrenheit { c * 1.8 + 32.0 } else { c };
        let _ = write!(s, "{:.0}{} · ", v, if m.fahrenheit { "°F" } else { "°C" });
    }
    if with_humidity && let Some(h) = m.here.humidity {
        let _ = write!(s, "{:.0}% · ", h);
    }
    let _ = s.push_str(air::summary(overall(&m.here)));
    s
}

/// lissajous burnin drift
fn drift(now: u64, ax: f32, ay: f32, period_s: f32) -> (i16, i16) {
    let t = now as f32 / 1000.0;
    (
        (ax * sinf(2.0 * PI * t / period_s)) as i16,
        (ay * cosf(2.0 * PI * t / (period_s * 1.37))) as i16,
    )
}

fn night(c: &mut Canvas, m: &Model, now: u64) {
    let (dx, dy) = drift(now, 24.0, 16.0, 600.0);
    let hm: String<8> = match m.time(now) {
        Some(t) => t.hhmm(m.clock_24h),
        None => {
            let mut s = String::new();
            let _ = s.push_str("--:--");
            s
        }
    };
    let w = style::NIGHT_CLOCK.width(&hm);
    c.text(
        Layer::Main,
        &style::NIGHT_CLOCK,
        &hm,
        400 - w / 2 + dx,
        268 + dy,
        Align::Left,
    );
    c.text(
        Layer::Main,
        &style::NIGHT_LINE,
        &conditions_line(m, false),
        400 + dx,
        318 + dy,
        Align::Centre,
    );
}

fn away(c: &mut Canvas, m: &Model, now: u64) {
    let t = now as f32 / 1000.0;
    let bx = 70 + (330.0 * (0.5 + 0.5 * sinf(2.0 * PI * t / 120.0))) as i16;
    let by = 70 + (190.0 * (0.5 + 0.5 * sinf(2.0 * PI * t / 70.0 + 1.0))) as i16;
    if let Some(time) = m.time(now) {
        let tw = c.text(
            Layer::Main,
            &style::AWAY_CLOCK,
            &time.hhmm(m.clock_24h),
            bx,
            by + 72,
            Align::Left,
        );
        if !m.clock_24h {
            c.text(
                Layer::Main,
                &style::AWAY_AMPM,
                time.ampm(),
                bx + tw + 8,
                by + 72,
                Align::Left,
            );
        }
    }
    c.text(
        Layer::Main,
        &style::AWAY_LINE,
        &conditions_line(m, true),
        bx + 4,
        by + 108,
        Align::Left,
    );
}

// system screen renderers

fn boot(c: &mut Canvas, m: &Model, now: u64) {
    c.sprite_at(
        Layer::Skin,
        sprite::BOOT_ORB_IDLE_0 + frame(now, 3200, 16),
        400,
        138,
    );
    const NAMES: [&str; 5] = [
        "Display",
        "Sensors",
        "Wi-Fi",
        "MQTT broker",
        "Home Assistant",
    ];
    for (i, (step, detail)) in m.boot.steps.iter().enumerate() {
        let y = 288 + 30 * i as i16;
        let (icon, name_style) = match step {
            Step::Pending => (sprite::BOOT_PENDING, &style::BOOT_STEP_DIM),
            Step::Running => (
                sprite::BOOT_SPIN_0 + frame(now, 1500, 12),
                &style::BOOT_STEP,
            ),
            Step::Done => (sprite::BOOT_CHECK, &style::BOOT_STEP),
            Step::Failed => (sprite::BOOT_PENDING, &style::BOOT_STEP),
        };
        c.sprite_at(Layer::Main, icon, 272, y - 6);
        c.text(Layer::Main, name_style, NAMES[i], 296, y, Align::Left);
        if matches!(step, Step::Done | Step::Failed) && !detail.is_empty() {
            c.text(
                Layer::Main,
                &style::BOOT_DETAIL,
                detail,
                450,
                y,
                Align::Left,
            );
        }
    }
    c.text(
        Layer::Main,
        &style::BOOT_VERSION,
        &m.boot.version,
        400,
        466,
        Align::Centre,
    );
}

fn setup<const N: usize>(c: &mut Canvas, x: &mut Extras<N>, m: &Model, now: u64) {
    let Some(s) = &m.setup else {
        return;
    };
    for (i, v) in [&s.ssid, &s.password, &s.url].into_iter().enumerate() {
        c.text(
            Layer::Main,
            &style::SETUP_VALUE,
            v,
            92,
            160 + 84 * i as i16 + 46,
            Align::Left,
        );
    }
    let dots = [
        "Waiting for a connection.",
        "Waiting for a connection..",
        "Waiting for a connection...",
    ];
    c.text(
        Layer::Main,
        &style::SETUP_WAIT,
        dots[(now / 500 % 3) as usize],
        40,
        452,
        Align::Left,
    );
    // padded qr payload
    let mut q: String<{ generated::QR_MAX_BYTES }> = String::new();
    let _ = write!(
        q,
        "WIFI:S:{};T:WPA;P:{};;",
        s.ssid.as_str(),
        s.password.as_str()
    );
    if !q.len().is_multiple_of(2) {
        let _ = q.push(';');
    }
    let mut vp = generated::QR_VP;
    for pair in q.as_bytes().chunks(2) {
        x.img.set(
            vp,
            u16::from_be_bytes([pair[0], *pair.get(1).unwrap_or(&b';')]),
        );
        vp += 1;
    }
    x.img.set(vp, 0xFFFF);
}

fn test(c: &mut Canvas, now: u64) {
    // perimeter test marker
    let (w, top, bottom) = (800i16, 340i16, 460i16);
    let perim = 2 * ((w - 20) + (bottom - top)) as u64;
    let p = ((now % 4000) * perim / 4000) as i16;
    let (mx, my) = if p < w - 20 {
        (p, top)
    } else if p < (w - 20) + (bottom - top) {
        (w - 20, top + (p - (w - 20)))
    } else if p < 2 * (w - 20) + (bottom - top) {
        (w - 20 - (p - (w - 20) - (bottom - top)), bottom)
    } else {
        (0, bottom - (p - 2 * (w - 20) - (bottom - top)))
    };
    c.sprite(
        Layer::Skin,
        sprite::TEST_MARKER,
        mx.clamp(0, w - 20),
        my.clamp(top, bottom),
    );
}

/// render active screen
pub fn render<const N: usize>(
    screen: Screen,
    c: &mut Canvas,
    x: &mut Extras<N>,
    m: &Model,
    now: u64,
) {
    match screen {
        Screen::Boot => boot(c, m, now),
        Screen::Home => home(c, x, m, now),
        Screen::Sensors => sensors(c, x, m, now),
        Screen::Rooms => rooms(c, m, now),
        Screen::Alert => alert(c, m, now),
        Screen::Night => night(c, m, now),
        Screen::Away => away(c, m, now),
        Screen::Setup => setup(c, x, m, now),
        Screen::Test => test(c, now),
    }
}

/// guess device icon
pub fn device_for(name: &str) -> Device {
    Device::guess(name)
}
