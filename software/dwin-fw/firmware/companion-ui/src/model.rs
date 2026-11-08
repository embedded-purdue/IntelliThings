//! screen data model

use heapless::{String, Vec};

use crate::air::State;

/// max message bytes
pub const MAX_TEXT: usize = 512;

/// node sensor readings
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Readings {
    pub temp_c: Option<f32>,
    pub humidity: Option<f32>,
    pub co2: Option<u16>,
    pub pm2_5: Option<u16>,
    pub voc: Option<u16>,
    pub lux: Option<f32>,
    pub presence: bool,
}

/// reading tile metrics
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Metric {
    Temp,
    Humidity,
    Co2,
    Pm2_5,
    Voc,
    Light,
}

impl Metric {
    pub const ALL: [Metric; 6] = [
        Metric::Temp,
        Metric::Humidity,
        Metric::Co2,
        Metric::Pm2_5,
        Metric::Voc,
        Metric::Light,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn label(self) -> &'static str {
        match self {
            Metric::Temp => "Temperature",
            Metric::Humidity => "Humidity",
            Metric::Co2 => "CO\u{2082}",
            Metric::Pm2_5 => "PM2.5",
            Metric::Voc => "VOC index",
            Metric::Light => "Light",
        }
    }
}

/// action device icons
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    Fan = 0,
    Bulb = 1,
    Thermostat = 2,
    Purifier = 3,
    Humidifier = 4,
    Window = 5,
    Door = 6,
    Lock = 7,
}

impl Device {
    /// guess device type
    pub fn guess(name: &str) -> Device {
        let has = |k: &str| contains_ignore_case(name, k);
        if has("light") || has("lamp") || has("bulb") {
            Device::Bulb
        } else if has("thermostat") || has("heat") || has("cool") || has("climate") {
            Device::Thermostat
        } else if has("purifier") || has("filter") {
            Device::Purifier
        } else if has("humidif") {
            Device::Humidifier
        } else if has("window") {
            Device::Window
        } else if has("door") {
            Device::Door
        } else if has("lock") {
            Device::Lock
        } else {
            Device::Fan
        }
    }
}

fn contains_ignore_case(hay: &str, needle: &str) -> bool {
    let n = needle.as_bytes();
    hay.as_bytes()
        .windows(n.len())
        .any(|w| w.eq_ignore_ascii_case(n))
}

/// action pill chip
#[derive(Clone, Debug, PartialEq)]
pub struct Chip {
    pub device: Device,
    pub label: String<28>,
    /// chip value string
    pub value: String<12>,
}

impl Chip {
    pub fn new(device: Device, label: &str, value: &str) -> Self {
        Self {
            device,
            label: truncated(label),
            value: truncated(value),
        }
    }
}

/// agent room statement
#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    pub text: String<MAX_TEXT>,
    /// arrival time ms
    pub at_ms: u64,
    /// payload unix time
    pub ts: Option<i64>,
    pub chips: Vec<Chip, 2>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Hub {
    #[default]
    Connecting,
    Online,
    Offline,
}

/// room status tile
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RoomStatus {
    pub online: bool,
    pub temp_c: Option<f32>,
    pub humidity: Option<f32>,
    pub state: Option<State>,
    pub word: &'static str,
    pub presence: bool,
    /// virtual room note
    pub note: String<16>,
}

/// takeover alert state
#[derive(Clone, Debug, PartialEq)]
pub struct Alert {
    pub title: String<40>,
    pub place: String<24>,
    pub since_ms: u64,
    pub metric: Metric,
    pub value: u16,
    pub normal: String<28>,
    pub message: String<160>,
    pub done: Vec<Chip, 2>,
    pub ask: Option<Chip>,
}

/// action toast banner
#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub device: Device,
    pub title: String<40>,
    pub detail: String<60>,
    pub at_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Step {
    #[default]
    Pending,
    Running,
    Done,
    Failed,
}

pub const BOOT_STEPS: usize = 5; // boot step count

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Boot {
    pub steps: [(Step, String<32>); BOOT_STEPS],
    pub version: String<40>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Setup {
    pub ssid: String<32>,
    pub password: String<32>,
    pub url: String<32>,
}

pub const TREND_POINTS: usize = crate::generated::TREND_POINTS;

/// sensor trend history
#[derive(Clone, Debug, PartialEq)]
pub struct Trends {
    pub values: [[f32; TREND_POINTS]; 6],
    pub len: usize,
}

impl Default for Trends {
    fn default() -> Self {
        Self {
            values: [[0.0; TREND_POINTS]; 6],
            len: 0,
        }
    }
}

impl Trends {
    /// push trend sample
    pub fn push(&mut self, r: &Readings) {
        let now = [
            r.temp_c,
            r.humidity,
            r.co2.map(f32::from),
            r.pm2_5.map(f32::from),
            r.voc.map(f32::from),
            r.lux,
        ];
        for (m, v) in now.iter().enumerate() {
            let row = &mut self.values[m];
            let last = if self.len > 0 {
                row[self.len.min(TREND_POINTS) - 1]
            } else {
                0.0
            };
            let v = v.unwrap_or(last);
            if self.len < TREND_POINTS {
                row[self.len] = v;
            } else {
                row.copy_within(1.., 0);
                row[TREND_POINTS - 1] = v;
            }
        }
        self.len = (self.len + 1).min(TREND_POINTS);
    }
}

/// full display model
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    /// current room name
    pub room: String<24>,
    /// room layout index
    pub room_index: usize,
    pub here: Readings,
    pub fahrenheit: bool,
    pub clock_24h: bool,
    /// wall clock tuple
    pub clock: Option<(i64, u64, i32)>,
    pub wifi: bool,
    pub hub: Hub,
    pub synced_ms: Option<u64>,
    pub retry_at_ms: Option<u64>,
    pub statement: Option<Statement>,
    /// agent thinking text
    pub thinking: Option<String<64>>,
    pub rooms: [RoomStatus; 5],
    pub trends: Trends,
    pub alert: Option<Alert>,
    pub toast: Option<Toast>,
    pub boot: Boot,
    pub setup: Option<Setup>,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            room: truncated("Living Room"),
            room_index: 0,
            here: Readings::default(),
            fahrenheit: true,
            clock_24h: false,
            clock: None,
            wifi: false,
            hub: Hub::Connecting,
            synced_ms: None,
            retry_at_ms: None,
            statement: None,
            thinking: None,
            rooms: Default::default(),
            trends: Trends::default(),
            alert: None,
            toast: None,
            boot: Boot::default(),
            setup: None,
        }
    }
}

impl Model {
    /// local wall time
    pub fn time(&self, now_ms: u64) -> Option<crate::clock::DateTime> {
        let (unix, at, offset) = self.clock?;
        let secs = unix + ((now_ms.saturating_sub(at)) / 1000) as i64;
        Some(crate::clock::DateTime::from_unix(secs, offset))
    }

    pub fn unix_now(&self, now_ms: u64) -> Option<i64> {
        let (unix, at, _) = self.clock?;
        Some(unix + ((now_ms.saturating_sub(at)) / 1000) as i64)
    }

    /// update statement text
    pub fn set_statement(&mut self, text: &str, ts: Option<i64>, now_ms: u64) {
        self.statement = Some(Statement {
            text: truncated(text),
            at_ms: now_ms,
            ts,
            chips: Vec::new(),
        });
        self.thinking = None;
    }
}

/// truncate string safely
pub fn truncated<const N: usize>(s: &str) -> String<N> {
    let mut out = String::new();
    for c in s.chars() {
        if out.push(c).is_err() {
            break;
        }
    }
    out
}
