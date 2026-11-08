//! dgus input memory

use heapless::{String, Vec};

use crate::air::State;
use crate::director::Settings;
use crate::model::{
    Alert, Chip, Device, Hub, MAX_TEXT, Metric, Model, Readings, RoomStatus, Setup, Statement,
    Step, Toast, truncated,
};
use crate::screens::{Screen, overall, shown};
pub use crate::vp_map::*;

/// display boot label
const DISPLAY_DETAIL: &str = "DMG80480T050 · 800×480";
/// trend sample interval
const TREND_EVERY_MS: u64 = 10 * 60 * 1000;

/// write reject reason
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejected {
    OutsideMap,
    ReadOnly,
}

/// display run mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// auto demo mode
    Auto,
    Live,
    Demo,
}

pub struct Inputs {
    mem: [u16; WORDS],
    /// synced timestamp ms
    synced_at: Option<u64>,
    retry_at: Option<u64>,
    clock_at: u64,
    toast_at: Option<u64>,
    alert_since_at: Option<u64>,
    trend_at: Option<u64>,
    /// live data flag
    data_written: bool,
    pub writes: u32,
    pub rejected: u32,
}

impl Default for Inputs {
    fn default() -> Self {
        Self::new()
    }
}

impl Inputs {
    pub const fn new() -> Self {
        let mut mem = [0u16; WORDS];
        let mut i = 0;
        while i < DEFAULTS.len() {
            mem[(DEFAULTS[i].0 - BASE) as usize] = DEFAULTS[i].1;
            i += 1;
        }
        mem[(BOOT0 + step::STATE - BASE) as usize] = 2;
        Self {
            mem,
            synced_at: None,
            retry_at: None,
            clock_at: 0,
            toast_at: None,
            alert_since_at: None,
            trend_at: None,
            data_written: false,
            writes: 0,
            rejected: 0,
        }
    }

    /// reset input memory
    pub fn reset(&mut self) {
        const STATUS_WORDS: usize = (WRITES_REJECTED - INTERFACE_VERSION + 1) as usize;
        let at = idx(INTERFACE_VERSION);
        let mut status = [0u16; STATUS_WORDS];
        status.copy_from_slice(&self.mem[at..at + STATUS_WORDS]);
        let (writes, rejected) = (self.writes, self.rejected);
        *self = Self::new();
        self.mem[at..at + STATUS_WORDS].copy_from_slice(&status);
        self.writes = writes;
        self.rejected = rejected;
    }

    fn get(&self, vp: u16) -> u16 {
        self.mem[idx(vp)]
    }

    /// write memory words
    pub fn write(&mut self, vp: u16, words: &[u16], now: u64) -> Result<(), Rejected> {
        let n = words.len();
        let inside = vp >= BASE && (vp - BASE) as usize + n <= WORDS && n > 0;
        let result = if !inside {
            Err(Rejected::OutsideMap)
        } else if READ_ONLY
            .iter()
            .any(|&(ro, len)| vp < ro + len as u16 && ro < vp + n as u16)
        {
            Err(Rejected::ReadOnly)
        } else {
            Ok(())
        };
        if result.is_err() {
            self.rejected = self.rejected.wrapping_add(1);
            self.mem[idx(WRITES_REJECTED)] = self.rejected as u16;
            return result;
        }
        let at = |v: u16| (v >= vp && v < vp + n as u16).then(|| words[(v - vp) as usize]);
        if at(INPUTS_RESET) == Some(RESET_MAGIC) {
            self.reset();
        } else {
            let old_trigger = self.get(TOAST_TRIGGER);
            let old_active = self.get(ALERT_ACTIVE);
            self.mem[idx(vp)..idx(vp) + n].copy_from_slice(words);
            if let Some(s) = at(SYNCED_AGO) {
                self.synced_at = (s != 0xFFFF).then(|| now.saturating_sub(s as u64 * 1000));
            }
            if let Some(s) = at(RETRY_IN) {
                self.retry_at = (s != 0).then(|| now + s as u64 * 1000);
            }
            if at(UNIX_TIME).is_some() || at(UNIX_TIME + 1).is_some() {
                self.clock_at = now;
            }
            if at(TOAST_TRIGGER).is_some_and(|t| t != old_trigger) {
                self.toast_at = Some(now);
            }
            if at(ALERT_ACTIVE).is_some_and(|a| a == 1 && old_active != 1) {
                self.alert_since_at = Some(now);
            }
            if let Some(s) = at(ALERT_SINCE) {
                self.alert_since_at = Some(now.saturating_sub(s as u64 * 1000));
            }
            // mode selection only
            if !(n == 1 && vp == MODE) {
                self.data_written = true;
            }
        }
        self.writes = self.writes.wrapping_add(1);
        self.mem[idx(WRITES_OK)] = self.writes as u16;
        Ok(())
    }

    /// read memory words
    pub fn read(&self, vp: u16, out: &mut [u16]) {
        for (k, w) in out.iter_mut().enumerate() {
            let v = vp.wrapping_add(k as u16);
            *w = if v >= BASE && ((v - BASE) as usize) < WORDS {
                self.get(v)
            } else {
                0
            };
        }
    }

    /// handle dgus request
    pub fn handle(
        &mut self,
        req: &Request,
        now: u64,
        reply: &mut [u8; REPLY_MAX],
    ) -> (usize, Option<Result<(), Rejected>>) {
        match req {
            Request::Write { vp, words } => {
                let r = self.write(*vp, words, now);
                reply[..6].copy_from_slice(&ACK);
                (6, Some(r))
            }
            Request::Read { vp, n } => {
                let n = (*n as usize).min(MAX_READ);
                let mut words = [0u16; MAX_READ];
                self.read(*vp, &mut words[..n]);
                reply[..7].copy_from_slice(&[
                    0x5A,
                    0xA5,
                    (4 + 2 * n) as u8,
                    0x83,
                    (vp >> 8) as u8,
                    *vp as u8,
                    n as u8,
                ]);
                for (k, w) in words[..n].iter().enumerate() {
                    reply[7 + 2 * k..9 + 2 * k].copy_from_slice(&w.to_be_bytes());
                }
                (7 + 2 * n, None)
            }
        }
    }

    /// update status words
    pub fn set_status(&mut self, panel_state: u16, screen: Screen, live: bool) {
        self.mem[idx(PANEL_STATE)] = panel_state;
        self.mem[idx(SCREEN_SHOWN)] = screen_code(screen);
        self.mem[idx(MODE_NOW)] = live as u16;
    }

    pub fn mode(&self) -> Mode {
        match self.get(MODE) {
            1 => Mode::Live,
            2 => Mode::Demo,
            _ => Mode::Auto,
        }
    }

    /// live mode active
    pub fn live(&self) -> bool {
        match self.mode() {
            Mode::Live => true,
            Mode::Demo => false,
            Mode::Auto => self.data_written,
        }
    }

    /// forced screen override
    pub fn screen(&self) -> Option<Screen> {
        match self.get(SCREEN) {
            1 => Some(Screen::Home),
            2 => Some(Screen::Sensors),
            3 => Some(Screen::Rooms),
            4 => Some(Screen::Alert),
            5 => Some(Screen::Night),
            6 => Some(Screen::Away),
            7 => Some(Screen::Boot),
            8 => Some(Screen::Setup),
            9 => Some(Screen::Test),
            _ => None,
        }
    }

    /// lightbar brightness cap
    pub fn led_brightness(&self) -> f32 {
        self.get(LED_BRIGHTNESS).min(100) as f32 / 100.0
    }

    /// sync model settings
    pub fn apply(&mut self, model: &mut Model, settings: &mut Settings, now: u64) {
        model.here = Readings {
            temp_c: self.signed(TEMPERATURE).map(|v| v as f32 / 10.0),
            humidity: self.reading(HUMIDITY).map(|v| v as f32 / 10.0),
            co2: self.reading(CO2),
            pm2_5: self.reading(PM2_5),
            voc: self.reading(VOC_INDEX),
            lux: self.reading(LUX).map(f32::from),
            // default presence true
            presence: self.get(PRESENCE) != 0,
        };
        let h = &model.here;
        let measured = h.temp_c.is_some()
            || h.humidity.is_some()
            || h.co2.is_some()
            || h.pm2_5.is_some()
            || h.voc.is_some()
            || h.lux.is_some();
        if measured
            && self
                .trend_at
                .is_none_or(|t| now.saturating_sub(t) >= TREND_EVERY_MS)
        {
            model.trends.push(&model.here);
            self.trend_at = Some(now);
        }

        model.wifi = self.get(WIFI) == 1;
        model.hub = match self.get(HUB) {
            1 => Hub::Online,
            2 => Hub::Offline,
            _ => Hub::Connecting,
        };
        model.synced_ms = self.synced_at;
        model.retry_at_ms = self.retry_at.filter(|&t| t > now);
        let unix = self.u32(UNIX_TIME);
        model.clock = (unix != 0).then(|| {
            (
                unix as i64,
                self.clock_at,
                self.get(UTC_OFFSET) as i16 as i32,
            )
        });

        model.fahrenheit = self.get(UNITS) != 1;
        model.clock_24h = self.get(CLOCK_24H) == 1;
        model.room_index = (self.get(ROOM) as usize).min(ROOM_NAMES.len() - 1);
        let name: String<24> = self.text(ROOM_NAME, ROOM_NAME_WORDS);
        model.room = if name.is_empty() {
            truncated(ROOM_NAMES[model.room_index])
        } else {
            name
        };
        let (start, end) = (self.get(NIGHT_START), self.get(NIGHT_END));
        (settings.night_start_min, settings.night_end_min) = if start == 0xFFFF || end == 0xFFFF {
            (0, 0) // never night
        } else {
            (start, end)
        };
        settings.away_after_ms = match self.get(AWAY_AFTER) {
            0 => u64::MAX,
            s => s as u64 * 1000,
        };
        settings.rotate_pages = self.get(ROTATE_PAGES) == 1;

        let text: String<MAX_TEXT> = self.text(MESSAGE_TEXT, MESSAGE_TEXT_WORDS);
        if text.is_empty() {
            model.statement = None;
        } else if model.statement.as_ref().is_none_or(|s| s.text != text) {
            model.statement = Some(Statement {
                text,
                at_ms: now,
                ts: None,
                chips: Vec::new(),
            });
            // clear thinking state
            self.mem[idx(THINKING)] = 0;
        }
        if let Some(s) = &mut model.statement {
            let ts = self.u32(MESSAGE_TS);
            s.ts = (ts != 0).then_some(ts as i64);
            s.chips = MESSAGE_CHIPS.iter().filter_map(|&c| self.chip(c)).collect();
        }
        let thinking: String<64> = self.text(THINKING, THINKING_WORDS);
        model.thinking = (!thinking.is_empty()).then_some(thinking);

        model.toast = self.toast_at.map(|at_ms| Toast {
            device: device(self.get(TOAST_DEVICE)),
            title: self.text(TOAST_TITLE, TOAST_TITLE_WORDS),
            detail: self.text(TOAST_DETAIL, TOAST_DETAIL_WORDS),
            at_ms,
        });

        model.alert = (self.get(ALERT_ACTIVE) == 1).then(|| {
            let metric = metric(self.get(ALERT_METRIC));
            Alert {
                title: self.text(ALERT_TITLE, ALERT_TITLE_WORDS),
                place: self.text(ALERT_PLACE, ALERT_PLACE_WORDS),
                since_ms: self.alert_since_at.unwrap_or(now),
                metric,
                value: shown_value(metric, self.get(ALERT_VALUE) as i16, model.fahrenheit),
                normal: self.text(ALERT_NORMAL, ALERT_NORMAL_WORDS),
                message: self.text(ALERT_MESSAGE, ALERT_MESSAGE_WORDS),
                done: [ALERT_DONE1, ALERT_DONE2]
                    .iter()
                    .filter_map(|&c| self.chip(c))
                    .collect(),
                ask: self.chip(ALERT_ASK),
            }
        });

        for (k, &base) in ROOMS.iter().enumerate() {
            let readings = match self.get(base + room::ONLINE) {
                1 => Some(Readings {
                    temp_c: self
                        .signed(base + room::TEMPERATURE)
                        .map(|v| v as f32 / 10.0),
                    humidity: self.reading(base + room::HUMIDITY).map(|v| v as f32 / 10.0),
                    co2: self.reading(base + room::CO2),
                    pm2_5: self.reading(base + room::PM2_5),
                    voc: self.reading(base + room::VOC_INDEX),
                    lux: None,
                    presence: self.get(base + room::PRESENCE) == 1,
                }),
                0xFFFF if k == model.room_index => Some(model.here),
                _ => None,
            };
            let mut r = RoomStatus {
                note: self.text(base + room::NOTE, room::NOTE_WORDS),
                ..Default::default()
            };
            if let Some(rd) = readings {
                (r.state, r.word) = room_judgement(&rd);
                r.online = true;
                r.temp_c = rd.temp_c;
                r.humidity = rd.humidity;
                r.presence = rd.presence;
            }
            model.rooms[k] = r;
        }

        // default completed steps
        let used = BOOT_STEPS[1..]
            .iter()
            .any(|&b| self.get(b + step::STATE) != 0xFFFF);
        for (k, &base) in BOOT_STEPS.iter().enumerate() {
            model.boot.steps[k] = if k == 0 {
                (Step::Done, truncated(DISPLAY_DETAIL))
            } else {
                let state = match self.get(base + step::STATE) {
                    _ if !used => Step::Done,
                    1 => Step::Running,
                    2 => Step::Done,
                    3 => Step::Failed,
                    _ => Step::Pending,
                };
                (state, self.text(base + step::DETAIL, step::DETAIL_WORDS))
            };
        }
        model.boot.version = self.text(BOOT_VERSION, BOOT_VERSION_WORDS);

        let ssid: String<32> = self.text(SETUP_SSID, SETUP_SSID_WORDS);
        model.setup = (!ssid.is_empty()).then(|| Setup {
            ssid,
            password: self.text(SETUP_PASSWORD, SETUP_PASSWORD_WORDS),
            url: self.text(SETUP_URL, SETUP_URL_WORDS),
        });
    }

    /// decode sensor reading
    fn reading(&self, vp: u16) -> Option<u16> {
        Some(self.get(vp)).filter(|&v| v < NO_READING)
    }

    fn signed(&self, vp: u16) -> Option<i16> {
        Some(self.get(vp))
            .filter(|&v| v != NO_READING)
            .map(|v| v as i16)
    }

    fn u32(&self, vp: u16) -> u32 {
        (self.get(vp) as u32) << 16 | self.get(vp + 1) as u32
    }

    /// decode text field
    fn text<const N: usize>(&self, vp: u16, words: usize) -> String<N> {
        let mut bytes = [0u8; MAX_TEXT];
        let mut len = 0;
        'words: for &w in &self.mem[idx(vp)..idx(vp) + words] {
            for b in w.to_be_bytes() {
                if b == 0 || b == 0xFF || len == bytes.len() {
                    break 'words;
                }
                bytes[len] = b;
                len += 1;
            }
        }
        let mut out = String::new();
        let mut rest = &bytes[..len];
        while !rest.is_empty() {
            let (good, bad) = match core::str::from_utf8(rest) {
                Ok(s) => (s, &[][..]),
                Err(e) => {
                    let (good, bad) = rest.split_at(e.valid_up_to());
                    let skip = e.error_len().unwrap_or(bad.len());
                    (core::str::from_utf8(good).unwrap_or(""), &bad[skip..])
                }
            };
            for c in good.chars() {
                if out.push(c).is_err() {
                    return out;
                }
            }
            rest = bad;
        }
        out
    }

    fn chip(&self, base: u16) -> Option<Chip> {
        let d = self.get(base + chip::DEVICE);
        (d != 0xFFFF).then(|| Chip {
            device: device(d),
            label: self.text(base + chip::LABEL, chip::LABEL_WORDS),
            value: self.text(base + chip::VALUE, chip::VALUE_WORDS),
        })
    }
}

const fn idx(vp: u16) -> usize {
    (vp - BASE) as usize
}

/// screen enum code
pub fn screen_code(s: Screen) -> u16 {
    match s {
        Screen::Home => 1,
        Screen::Sensors => 2,
        Screen::Rooms => 3,
        Screen::Alert => 4,
        Screen::Night => 5,
        Screen::Away => 6,
        Screen::Boot => 7,
        Screen::Setup => 8,
        Screen::Test => 9,
    }
}

fn device(code: u16) -> Device {
    match code {
        1 => Device::Bulb,
        2 => Device::Thermostat,
        3 => Device::Purifier,
        4 => Device::Humidifier,
        5 => Device::Window,
        6 => Device::Door,
        7 => Device::Lock,
        _ => Device::Fan,
    }
}

fn metric(code: u16) -> Metric {
    Metric::ALL
        .get(code as usize)
        .copied()
        .unwrap_or(Metric::Pm2_5)
}

/// format alert value
fn shown_value(m: Metric, raw: i16, fahrenheit: bool) -> u16 {
    let v = match m {
        Metric::Temp if fahrenheit => raw as f32 / 10.0 * 1.8 + 32.0,
        Metric::Temp | Metric::Humidity => raw as f32 / 10.0,
        _ => raw as f32,
    };
    (v + 0.5).max(0.0) as u16
}

/// evaluate room air
pub fn room_judgement(r: &Readings) -> (Option<State>, &'static str) {
    let state = overall(r);
    if state == State::Neutral {
        return (None, ""); // no readings yet
    }
    if !state.is_alert() {
        return (Some(state), "Air fresh");
    }
    let word = [
        Metric::Temp,
        Metric::Humidity,
        Metric::Co2,
        Metric::Pm2_5,
        Metric::Voc,
    ]
    .iter()
    .map(|&m| shown(m, r, false).j)
    .find(|j| j.state == state)
    .map_or("", |j| j.word);
    (Some(state), word)
}

// serial frame decoder

/// write reply ack
pub const ACK: [u8; 6] = [0x5A, 0xA5, 0x03, 0x82, 0x4F, 0x4B];
/// max read words
pub const MAX_READ: usize = 125;
/// max reply bytes
pub const REPLY_MAX: usize = 7 + 2 * MAX_READ;

/// node request frame
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    clippy::large_enum_variant,
    reason = "one per received frame, and no allocator to box it"
)]
pub enum Request {
    Write { vp: u16, words: Vec<u16, 126> },
    Read { vp: u16, n: u8 },
}

/// dgus stream parser
pub struct Parser {
    buf: [u8; 3 + 255],
    len: usize,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub const fn new() -> Self {
        Self {
            buf: [0; 3 + 255],
            len: 0,
        }
    }

    /// drop partial frame
    pub fn reset(&mut self) {
        self.len = 0;
    }

    /// push stream byte
    pub fn push(&mut self, byte: u8) -> Option<Request> {
        match self.len {
            0 if byte != 0x5A => return None,
            1 if byte != 0xA5 => {
                self.len = (byte == 0x5A) as usize;
                return None;
            }
            2 if byte < 4 => {
                self.len = 0;
                return None;
            }
            _ => {}
        }
        self.buf[self.len] = byte;
        self.len += 1;
        if self.len < 3 || self.len < 3 + self.buf[2] as usize {
            return None;
        }
        let frame = &self.buf[3..self.len];
        self.len = 0;
        let vp = u16::from_be_bytes([frame[1], frame[2]]);
        match frame[0] {
            0x82 if frame.len() % 2 == 1 => {
                let words = frame[3..]
                    .chunks(2)
                    .map(|p| u16::from_be_bytes([p[0], p[1]]))
                    .collect();
                Some(Request::Write { vp, words })
            }
            0x83 if frame.len() == 4 => Some(Request::Read { vp, n: frame[3] }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::director::Director;
    use std::vec::Vec as StdVec;

    fn frame(vp: u16, words: &[u16]) -> StdVec<u8> {
        let mut f = vec![0x5A, 0xA5, (3 + 2 * words.len()) as u8, 0x82];
        f.extend_from_slice(&vp.to_be_bytes());
        for w in words {
            f.extend_from_slice(&w.to_be_bytes());
        }
        f
    }

    fn text_words(s: &str) -> StdVec<u16> {
        let mut b = s.as_bytes().to_vec();
        b.push(0);
        if b.len() % 2 == 1 {
            b.push(0);
        }
        b.chunks(2)
            .map(|p| u16::from_be_bytes([p[0], p[1]]))
            .collect()
    }

    fn feed(inputs: &mut Inputs, bytes: &[u8], now: u64) -> StdVec<u8> {
        let mut parser = Parser::new();
        let mut out = StdVec::new();
        let mut reply = [0u8; REPLY_MAX];
        for &b in bytes {
            if let Some(req) = parser.push(b) {
                let (n, _) = inputs.handle(&req, now, &mut reply);
                out.extend_from_slice(&reply[..n]);
            }
        }
        out
    }

    fn applied(inputs: &mut Inputs, model: &mut Model, now: u64) -> Settings {
        let mut s = Settings::default();
        inputs.apply(model, &mut s, now);
        s
    }

    #[test]
    fn power_up_shows_nothing_yet() {
        let mut i = Inputs::new();
        let mut m = Model::default();
        applied(&mut i, &mut m, 0);
        assert_eq!(m.here.temp_c, None);
        assert!(m.here.presence, "no presence sensor must not mean nobody");
        assert_eq!(
            (m.hub, m.statement.is_none(), m.alert.is_none()),
            (Hub::Connecting, true, true)
        );
        assert!(!i.live(), "auto mode plays the demo until data arrives");
        // direct home transition
        let mut d = Director::new(Settings::default());
        assert_eq!(d.decide(&m, 0).screen, Screen::Home);
    }

    #[test]
    fn sensor_block_in_one_frame() {
        let mut i = Inputs::new();
        let replies = feed(
            &mut i,
            &frame(TEMPERATURE, &[224, 412, 118, 642, 320, 1180, 3, 6, 7, 1]),
            0,
        );
        assert_eq!(replies, ACK);
        let mut m = Model::default();
        applied(&mut i, &mut m, 0);
        assert_eq!(m.here.temp_c, Some(22.4));
        assert_eq!(m.here.humidity, Some(41.2));
        assert_eq!(
            (m.here.voc, m.here.co2, m.here.pm2_5),
            (Some(118), Some(642), Some(6))
        );
        assert_eq!(m.here.lux, Some(320.0));
        assert!(i.live());
        // negative and missing
        feed(&mut i, &frame(TEMPERATURE, &[(-50i16) as u16]), 0);
        feed(&mut i, &frame(PM2_5, &[NO_READING]), 0);
        applied(&mut i, &mut m, 0);
        assert_eq!((m.here.temp_c, m.here.pm2_5), (Some(-5.0), None));
    }

    #[test]
    fn messages_change_only_when_the_text_does() {
        let mut i = Inputs::new();
        let mut m = Model::default();
        feed(&mut i, &frame(THINKING, &text_words("Looking…")), 0);
        applied(&mut i, &mut m, 0);
        assert_eq!(m.thinking.as_deref(), Some("Looking…"));

        feed(&mut i, &frame(MESSAGE_TEXT, &text_words("CO₂ is up.")), 100);
        applied(&mut i, &mut m, 100);
        let s = m.statement.clone().unwrap();
        assert_eq!((s.text.as_str(), s.at_ms), ("CO₂ is up.", 100));
        assert_eq!(m.thinking, None, "a new message ends the thinking line");

        // retained message rewrite
        feed(
            &mut i,
            &frame(MESSAGE_TEXT, &text_words("CO₂ is up.")),
            5000,
        );
        applied(&mut i, &mut m, 5000);
        assert_eq!(m.statement.as_ref().unwrap().at_ms, 100);

        feed(&mut i, &frame(MESSAGE_TEXT, &[0]), 6000);
        applied(&mut i, &mut m, 6000);
        assert_eq!(m.statement, None);
    }

    #[test]
    fn long_text_in_three_frames() {
        let text: std::string::String = "Fresh air. ".repeat(40); // test message bytes
        let words = text_words(&text);
        let mut i = Inputs::new();
        let mut bytes = StdVec::new();
        let mut frames = 0;
        for (k, chunk) in words.chunks(120).enumerate() {
            bytes.extend(frame(MESSAGE_TEXT + 120 * k as u16, chunk));
            frames += 1;
        }
        assert_eq!(frames, 2);
        assert_eq!(
            feed(&mut i, &bytes, 0),
            ACK.repeat(frames),
            "one OK per frame"
        );
        let mut m = Model::default();
        applied(&mut i, &mut m, 0);
        assert_eq!(m.statement.unwrap().text.as_str(), text);
    }

    #[test]
    fn chips_toast_and_alert() {
        let mut i = Inputs::new();
        let mut m = Model::default();
        feed(&mut i, &frame(MESSAGE_TEXT, &text_words("Fan up.")), 0);
        feed(&mut i, &frame(CHIP1, &[0]), 0);
        feed(&mut i, &frame(CHIP1 + chip::LABEL, &text_words("Fan")), 0);
        feed(&mut i, &frame(CHIP1 + chip::VALUE, &text_words("40%")), 0);
        let mut toast = vec![1, 1];
        toast.extend(text_words("Lights dimmed"));
        feed(&mut i, &frame(TOAST_TRIGGER, &toast), 1000);
        feed(&mut i, &frame(ALERT_ACTIVE, &[1, 3, 87, 120]), 2000);
        applied(&mut i, &mut m, 2000);
        let chips = &m.statement.as_ref().unwrap().chips;
        assert_eq!(chips.len(), 1);
        assert_eq!(
            (
                chips[0].device,
                chips[0].label.as_str(),
                chips[0].value.as_str()
            ),
            (Device::Fan, "Fan", "40%")
        );
        let t = m.toast.as_ref().unwrap();
        assert_eq!(
            (t.device, t.title.as_str(), t.at_ms),
            (Device::Bulb, "Lights dimmed", 1000)
        );
        let a = m.alert.as_ref().unwrap();
        assert_eq!((a.metric, a.value, a.since_ms), (Metric::Pm2_5, 87, 0));
        // toast trigger change
        feed(&mut i, &frame(TOAST_TRIGGER, &[1]), 9000);
        applied(&mut i, &mut m, 9000);
        assert_eq!(m.toast.as_ref().unwrap().at_ms, 1000);
        feed(&mut i, &frame(TOAST_TRIGGER, &[2]), 9000);
        applied(&mut i, &mut m, 9000);
        assert_eq!(m.toast.as_ref().unwrap().at_ms, 9000);
    }

    #[test]
    fn temperature_alerts_follow_the_units() {
        assert_eq!(shown_value(Metric::Temp, 300, true), 86);
        assert_eq!(shown_value(Metric::Temp, 300, false), 30);
        assert_eq!(shown_value(Metric::Humidity, 712, true), 71);
        assert_eq!(shown_value(Metric::Co2, 1250, true), 1250);
    }

    #[test]
    fn rooms_judge_themselves() {
        let mut i = Inputs::new();
        let mut m = Model::default();
        feed(&mut i, &frame(TEMPERATURE, &[222, 410, 100, 600, 300]), 0);
        // room reading fixture
        feed(&mut i, &frame(ROOM2, &[1, 233, 480, 700, 40, 120, 1]), 0);
        feed(
            &mut i,
            &frame(ROOM4 + room::NOTE, &text_words("Lights on")),
            0,
        );
        applied(&mut i, &mut m, 0);
        let own = &m.rooms[0];
        assert!(
            own.online && own.temp_c == Some(22.2),
            "this node's room fills itself"
        );
        assert_eq!((own.state, own.word), (Some(State::Good), "Air fresh"));
        let k = &m.rooms[1];
        assert_eq!(
            (k.state, k.word, k.presence),
            (Some(State::Poor), "Poor", true)
        );
        assert!(!m.rooms[2].online);
        assert_eq!(m.rooms[3].note.as_str(), "Lights on");
    }

    #[test]
    fn every_room_word_can_be_drawn() {
        use crate::generated::style;
        // test band coverage
        let samples = [
            0.0f32, 10.0, 17.0, 22.0, 26.0, 30.0, 40.0, 65.0, 80.0, 200.0, 300.0, 450.0, 900.0,
            1500.0, 2500.0,
        ];
        let mut words = vec!["Air fresh"];
        for v in samples {
            let r = Readings {
                temp_c: Some(v),
                humidity: Some(v),
                co2: Some(v as u16),
                pm2_5: Some(v as u16),
                voc: Some(v as u16),
                ..Default::default()
            };
            for m in [
                Metric::Temp,
                Metric::Humidity,
                Metric::Co2,
                Metric::Pm2_5,
                Metric::Voc,
            ] {
                words.push(shown(m, &r, false).j.word);
            }
        }
        for w in words {
            for st in [
                &style::ROOM_WORD_GOOD,
                &style::ROOM_WORD_MODERATE,
                &style::ROOM_WORD_POOR,
                &style::ROOM_WORD_BAD,
            ] {
                assert!(
                    w.chars().all(|c| st.index(c).is_some()),
                    "{w:?} has a character the rooms page can't draw"
                );
            }
        }
    }

    #[test]
    fn boot_steps_show_once_written() {
        let mut i = Inputs::new();
        let mut m = Model::default();
        feed(&mut i, &frame(BOOT1, &[1]), 0);
        applied(&mut i, &mut m, 0);
        let states: StdVec<Step> = m.boot.steps.iter().map(|s| s.0).collect();
        assert_eq!(
            states,
            [
                Step::Done,
                Step::Running,
                Step::Pending,
                Step::Pending,
                Step::Pending
            ]
        );
        let mut d = Director::new(Settings::default());
        assert_eq!(d.decide(&m, 0).screen, Screen::Boot);
        for b in &BOOT_STEPS[1..] {
            feed(&mut i, &frame(*b, &[2]), 0);
        }
        applied(&mut i, &mut m, 0);
        assert_eq!(
            d.decide(&m, 10_000).screen,
            Screen::Boot,
            "held a moment to read"
        );
        assert_eq!(d.decide(&m, 12_000).screen, Screen::Home);
    }

    #[test]
    fn time_relative_words_count_from_when_written() {
        let mut i = Inputs::new();
        let mut m = Model::default();
        feed(&mut i, &frame(WIFI, &[1, 2, 30, 42]), 100_000);
        feed(
            &mut i,
            &frame(UNIX_TIME, &[0x6AD5, 0x0A4C, (-240i16) as u16]),
            100_000,
        );
        applied(&mut i, &mut m, 101_000);
        assert_eq!((m.wifi, m.hub), (true, Hub::Offline));
        assert_eq!(m.synced_ms, Some(70_000));
        assert_eq!(m.retry_at_ms, Some(142_000));
        let t = m.time(101_000).unwrap();
        assert_eq!((t.hour, t.minute), (14, 5));
    }

    #[test]
    fn read_only_words_and_the_map_edges() {
        let mut i = Inputs::new();
        assert_eq!(i.write(INTERFACE_VERSION, &[7], 0), Err(Rejected::ReadOnly));
        assert_eq!(i.write(0x0084, &[0x5A01, 1], 0), Err(Rejected::OutsideMap));
        assert_eq!(
            i.write(BASE + WORDS as u16 - 1, &[1, 2], 0),
            Err(Rejected::OutsideMap)
        );
        assert_eq!(i.write(BOOT0, &[2], 0), Err(Rejected::ReadOnly));
        assert_eq!(i.rejected, 4);
        let replies = feed(&mut i, &[0x5A, 0xA5, 0x04, 0x83, 0x10, 0xF0, 0x06], 0);
        assert_eq!(&replies[..7], &[0x5A, 0xA5, 0x10, 0x83, 0x10, 0xF0, 0x06]);
        assert_eq!(&replies[7..9], &VERSION.to_be_bytes());
        assert_eq!(&replies[17..19], &[0, 4], "writes_rejected");
    }

    #[test]
    fn reset_restores_power_up_values() {
        let mut i = Inputs::new();
        i.write(CO2, &[900], 0).unwrap();
        i.write(ALERT_ACTIVE, &[1], 0).unwrap();
        i.write(INPUTS_RESET, &[RESET_MAGIC], 0).unwrap();
        let mut m = Model::default();
        applied(&mut i, &mut m, 0);
        assert_eq!((m.here.co2, m.alert.is_none()), (None, true));
        assert!(!i.live(), "after a reset the demo plays until new data");
        let mut w = [0u16; 1];
        i.read(WRITES_OK, &mut w);
        assert_eq!(w[0], 3, "the counters survive a reset");
    }

    #[test]
    fn parser_resyncs_and_splits() {
        let mut p = Parser::new();
        let mut got = StdVec::new();
        let mut bytes = vec![0x00, 0x5A, 0x5A, 0xA5, 0x05, 0x82, 0x10, 0x03, 0x03];
        bytes.extend([
            0x2C, 0x5A, 0xA5, 0x02, 0x5A, 0xA5, 0x04, 0x83, 0x10, 0x00, 0x0A,
        ]);
        for b in bytes {
            got.extend(p.push(b));
        }
        let mut words = Vec::new();
        words.push(812).unwrap();
        assert_eq!(
            got,
            [
                Request::Write { vp: CO2, words },
                Request::Read { vp: 0x1000, n: 10 }
            ]
        );
    }

    #[test]
    fn text_is_cut_at_a_character_and_skips_garbage() {
        let mut i = Inputs::new();
        // invalid utf8 sequence
        i.write(TOAST_TITLE, &[0x61C3, 0x2862, 0x0000], 0).unwrap();
        let s: String<40> = i.text(TOAST_TITLE, TOAST_TITLE_WORDS);
        assert_eq!(s.as_str(), "a(b");
        let s: String<3> = i.text(TOAST_TITLE, TOAST_TITLE_WORDS);
        assert_eq!(s.as_str(), "a(b");
        i.write(TOAST_TITLE, &text_words("°°°"), 0).unwrap();
        let s: String<5> = i.text(TOAST_TITLE, TOAST_TITLE_WORDS);
        assert_eq!(
            s.as_str(),
            "°°",
            "a two-byte character that doesn't fit is left out"
        );
    }

    /// test decoder match
    #[test]
    fn team_helpers_round_trip() {
        use crate::dwin_vp as team;
        assert_eq!(team::CO2, CO2);
        assert_eq!(team::ROOMS[1] + team::room::NOTE, ROOM2 + room::NOTE);
        let mut buf = [0u8; 6 + team::MAX_DATA];
        let n = team::write_frame(&mut buf, team::CO2, &[812]);
        assert_eq!(&buf[..n], &frame(CO2, &[812])[..]);
        let mut words = [0u16; team::MESSAGE_TEXT_WORDS];
        let k = team::text_words("CO₂ is up.", &mut words);
        let mut i = Inputs::new();
        i.write(team::MESSAGE_TEXT, &words[..k], 0).unwrap();
        let s: String<64> = i.text(MESSAGE_TEXT, MESSAGE_TEXT_WORDS);
        assert_eq!(s.as_str(), "CO₂ is up.");
        let mut small = [0u16; 2]; // short text buffer
        let k = team::text_words("CO₂", &mut small);
        assert_eq!(&small[..k], &[0x434F, 0x0000]);
        assert_eq!(team::x10(Some(22.44)), 224);
        assert_eq!(team::x10(Some(-5.0)), (-50i16) as u16);
        assert_eq!(team::x10(None), NO_READING);
    }
}
