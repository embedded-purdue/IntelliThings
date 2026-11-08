//! demo scenarios

use libm::expf;

use crate::air::State;
use crate::clock::parse_iso8601;
use crate::model::{
    Alert, Chip, Device, Hub, Metric, Model, Readings, RoomStatus, Setup, Statement, Step, Toast,
    truncated,
};
use crate::screens::Screen;

/// scenario showcase list
pub const SCENARIOS: [(&str, u32, f32); 13] = [
    ("home", 6, 2.0),
    ("co2-rising", 12, 9.5),
    ("agent-thinking", 12, 7.8),
    ("action-toast", 8, 3.0),
    ("sensors", 6, 3.0),
    ("rooms", 6, 2.0),
    ("alert", 8, 1.6),
    ("offline", 8, 2.0),
    ("night", 8, 1.0),
    ("away", 12, 3.0),
    ("status-boot", 8, 6.5),
    ("setup-wifi", 6, 1.0),
    ("test-pattern", 4, 0.0),
];

/// loop duration ms
pub fn loop_ms() -> u64 {
    SCENARIOS.iter().map(|s| s.1 as u64 * 1000).sum()
}

/// scenario at time
pub fn at(ms: u64) -> (&'static str, u64) {
    let mut t = ms % loop_ms();
    for (name, secs, _) in SCENARIOS {
        let len = secs as u64 * 1000;
        if t < len {
            return (name, t);
        }
        t -= len;
    }
    (SCENARIOS[0].0, 0)
}

fn base(now: u64) -> Model {
    let mut m = Model::default();
    for s in m.boot.steps.iter_mut() {
        s.0 = Step::Done;
    }
    m.hub = Hub::Online;
    m.wifi = true;
    m.synced_ms = Some(now.saturating_sub(120_000));
    m.here = Readings {
        temp_c: Some((72.4 - 32.0) / 1.8),
        humidity: Some(41.0),
        co2: Some(642),
        pm2_5: Some(6),
        voc: Some(118),
        lux: Some(320.0),
        presence: true,
    };
    m.clock = Some((
        parse_iso8601("2026-10-04T19:42:00-04:00").unwrap_or(0),
        now,
        -240,
    ));
    m
}

fn statement(text: &str, at_ms: u64, chips: &[Chip]) -> Option<Statement> {
    let mut c = heapless::Vec::new();
    for ch in chips {
        let _ = c.push(ch.clone());
    }
    Some(Statement {
        text: truncated(text),
        at_ms,
        ts: None,
        chips: c,
    })
}

/// trend random walk
fn series(
    seed: u32,
    base: f32,
    spread: f32,
    drift: f32,
    spike_at: Option<usize>,
    spike: f32,
) -> [f32; 72] {
    let mut out = [0.0; 72];
    let mut v = 0.0f32;
    let mut x = seed.wrapping_mul(2_654_435_761).wrapping_add(12_345);
    for (i, o) in out.iter_mut().enumerate() {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        let r = (x % 2000) as f32 / 1000.0 - 1.0;
        v = v * 0.85 + r * 0.5;
        let bump = spike_at.map_or(0.0, |at| {
            let d = (i as f32 - at as f32) / 4.0;
            spike * expf(-d * d)
        });
        *o = base + v * spread + drift * i as f32 / 72.0 + bump;
    }
    out
}

const CALM: &str = "Everything looks comfortable. Air is fresh, and the room is holding at 72°F.";

/// build scenario frame
pub fn scenario(name: &str, t: u64, now: u64) -> Option<(Screen, Model)> {
    let start = now.saturating_sub(t);
    let mut m = base(now);
    let screen = match name {
        "home" => {
            m.statement = statement(CALM, now.saturating_sub(120_000), &[]);
            Screen::Home
        }
        "co2-rising" => {
            m.here.co2 = Some((1040 + t.min(6000) * 140 / 6000) as u16);
            m.synced_ms = Some(start);
            if t < 3000 {
                m.statement = statement(CALM, start.saturating_sub(300_000), &[]);
            } else if t < 4500 {
                m.thinking = Some(truncated("Looking at the air in here…"));
            } else {
                m.statement = statement(
                    "CO₂ hit 1,180 ppm, so I turned the fan up to 40%. Opening a window would help too.",
                    start + 4500,
                    &[Chip::new(Device::Fan, "Fan", "40%")],
                );
            }
            Screen::Home
        }
        "agent-thinking" => {
            m.synced_ms = Some(if t < 4500 {
                start.saturating_sub(300_000)
            } else {
                start + 4500
            });
            if t < 1500 {
                m.statement = statement(CALM, start.saturating_sub(300_000), &[]);
            } else if t < 4500 {
                m.thinking = Some(truncated("Reading 6 sensors across 3 rooms…"));
            } else {
                m.statement = statement(
                    "The empty bedroom hit 75°F, so I nudged its thermostat down 2°.",
                    start + 4500,
                    &[Chip::new(Device::Thermostat, "Bedroom", "70°F")],
                );
            }
            Screen::Home
        }
        "action-toast" => {
            m.here.lux = Some(85.0);
            m.statement = statement(
                "It's after sunset and someone's here, so I warmed and dimmed the lights.",
                start.saturating_sub(1000),
                &[Chip::new(Device::Bulb, "Lights", "30%")],
            );
            m.toast = Some(Toast {
                device: Device::Bulb,
                title: truncated("Lights dimmed to 30%"),
                detail: truncated("Living Room · after sunset, someone's here"),
                at_ms: start + 1000,
            });
            Screen::Home
        }
        "sensors" => {
            m.here.co2 = Some(842);
            m.trends.values = [
                series(1, 71.5, 0.6, 1.0, None, 0.0),
                series(2, 43.0, 1.6, -2.0, None, 0.0),
                series(3, 540.0, 30.0, 300.0, None, 0.0),
                series(4, 5.0, 1.2, 0.0, Some(50), 22.0),
                series(5, 110.0, 14.0, 10.0, Some(50), 120.0),
                series(6, 380.0, 30.0, -120.0, None, 0.0),
            ];
            m.trends.len = 72;
            Screen::Sensors
        }
        "rooms" => {
            let room = |t: f32, h: f32, s: State, w: &'static str, p: bool| RoomStatus {
                online: true,
                temp_c: Some(t),
                humidity: Some(h),
                state: Some(s),
                word: w,
                presence: p,
                note: truncated(""),
            };
            m.rooms[0] = room(22.2, 41.0, State::Good, "Air fresh", true);
            m.rooms[1] = room(23.3, 48.0, State::Moderate, "Particles up", true);
            m.rooms[2] = room(23.9, 39.0, State::Good, "Air fresh", false);
            m.rooms[4].note = truncated("Lights on");
            m.synced_ms = Some(now.saturating_sub(60_000));
            Screen::Rooms
        }
        "alert" => {
            let mut done = heapless::Vec::new();
            let _ = done.push(Chip::new(Device::Fan, "Range hood", "High"));
            let _ = done.push(Chip::new(Device::Purifier, "Purifier", "Turbo"));
            m.alert = Some(Alert {
                title: truncated("Smoke or cooking particles"),
                place: truncated("Kitchen"),
                since_ms: now.saturating_sub(120_000),
                metric: Metric::Pm2_5,
                value: 87,
                normal: truncated("Normal is under 12"),
                message: truncated("Probably the stove. I'll tell you when the air clears."),
                done,
                ask: Some(Chip::new(Device::Door, "Close the bedroom door", "")),
            });
            Screen::Alert
        }
        "offline" => {
            m.hub = Hub::Offline;
            m.retry_at_ms = Some(start + 42_000);
            Screen::Home
        }
        "night" => {
            m.clock = Some((
                parse_iso8601("2026-10-04T23:48:00-04:00").unwrap_or(0),
                now,
                -240,
            ));
            m.here.temp_c = Some(20.0);
            Screen::Night
        }
        "away" => {
            m.clock = Some((
                parse_iso8601("2026-10-04T19:58:00-04:00").unwrap_or(0),
                now,
                -240,
            ));
            if t < 10_500 {
                m.here.presence = false;
                Screen::Away
            } else {
                m.statement = statement(CALM, now.saturating_sub(120_000), &[]);
                Screen::Home
            }
        }
        "status-boot" => {
            let details = [
                "DMG80480T050 · 800×480",
                "6 of 6 responding",
                "HomeNet · −52 dBm",
                "intellithings/living-room",
                "MCP ready",
            ];
            for (i, d) in details.iter().enumerate() {
                let begin = 600 + 1100 * i as u64;
                let step = if t < begin {
                    Step::Pending
                } else if t < begin + 900 {
                    Step::Running
                } else {
                    Step::Done
                };
                m.boot.steps[i] = (step, truncated(d));
            }
            m.boot.version = truncated("v0.1.0 · node living-room-01");
            Screen::Boot
        }
        "setup-wifi" => {
            m.setup = Some(Setup {
                ssid: truncated("IntelliThings-3F2A"),
                password: truncated("48213907"),
                url: truncated("192.168.4.1"),
            });
            Screen::Setup
        }
        "test-pattern" => Screen::Test,
        _ => return None,
    };
    Some((screen, m))
}
