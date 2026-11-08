//! screen pattern director

use core::mem::discriminant;

use crate::air::State;
use crate::lightbar::{self, Pattern, Rgb};
use crate::model::{Hub, Model, Step};
use crate::screens::{Screen, overall};

#[derive(Clone, Copy, Debug)]
pub struct Settings {
    /// night start minute
    pub night_start_min: u16,
    pub night_end_min: u16,
    /// away timeout ms
    pub away_after_ms: u64,
    /// unread breathe ms
    pub unread_ms: u64,
    /// rotate pages flag
    pub rotate_pages: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            night_start_min: 22 * 60 + 30,
            night_end_min: 6 * 60 + 30,
            away_after_ms: 10 * 60 * 1000,
            unread_ms: 2 * 60 * 1000,
            rotate_pages: false,
        }
    }
}

pub struct Director {
    pub settings: Settings,
    last_presence_ms: u64,
    away: bool,
    greeting_at: Option<u64>,
    was_booting: bool,
    boot_done_at: Option<u64>,
    clock: PatternClock,
}

/// pattern timing clock
#[derive(Default)]
pub struct PatternClock {
    current: Option<(Pattern, u64)>,
}

impl PatternClock {
    /// update pattern elapsed
    pub fn update(&mut self, pattern: Pattern, fixed_start: Option<u64>, now: u64) -> f32 {
        let start = match (self.current, fixed_start) {
            (_, Some(s)) => s,
            (Some((p, s)), None) if discriminant(&p) == discriminant(&pattern) => s,
            _ => now,
        };
        self.current = Some((pattern, start));
        now.saturating_sub(start) as f32 / 1000.0
    }
}

/// ui display decision
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decision {
    pub screen: Screen,
    pub pattern: Pattern,
    /// pattern elapsed seconds
    pub pattern_t: f32,
}

const GREETING_MS: u64 = 4200;
const BOOT_HOLD_MS: u64 = 1500;

impl Director {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            last_presence_ms: 0,
            away: false,
            greeting_at: None,
            was_booting: false,
            boot_done_at: None,
            clock: PatternClock::default(),
        }
    }

    pub fn decide(&mut self, m: &Model, now: u64) -> Decision {
        // update presence state
        if m.here.presence {
            if self.away {
                self.greeting_at = Some(now);
            }
            self.away = false;
            self.last_presence_ms = now;
        } else if now.saturating_sub(self.last_presence_ms) >= self.settings.away_after_ms {
            self.away = true;
        }
        // boot visibility window
        let booting = m
            .boot
            .steps
            .iter()
            .any(|(s, _)| matches!(s, Step::Pending | Step::Running));
        if booting {
            self.was_booting = true;
        } else if self.was_booting {
            self.was_booting = false;
            self.boot_done_at = Some(now);
        }
        let boot_visible = booting
            || self
                .boot_done_at
                .is_some_and(|t| now.saturating_sub(t) < BOOT_HOLD_MS);

        let night = m.time(now).is_some_and(|t| {
            let min = t.hour as u16 * 60 + t.minute as u16;
            let (a, b) = (self.settings.night_start_min, self.settings.night_end_min);
            if a <= b {
                (a..b).contains(&min)
            } else {
                min >= a || min < b
            }
        });

        let screen = if m.setup.is_some() {
            Screen::Setup
        } else if boot_visible {
            Screen::Boot
        } else if m.alert.is_some() {
            Screen::Alert
        } else if night {
            Screen::Night
        } else if self.away {
            Screen::Away
        } else if self.settings.rotate_pages {
            match (now / 1000) % 60 {
                0..40 => Screen::Home,
                40..50 => Screen::Sensors,
                _ => Screen::Rooms,
            }
        } else {
            Screen::Home
        };

        let (pattern, start) =
            pattern_for(screen, m, now, self.greeting_at, self.settings.unread_ms);
        let pattern_t = self.clock.update(pattern, start, now);
        Decision {
            screen,
            pattern,
            pattern_t,
        }
    }
}

/// resolve lightbar pattern
pub fn pattern_for(
    screen: Screen,
    m: &Model,
    now: u64,
    greeting_at: Option<u64>,
    unread_ms: u64,
) -> (Pattern, Option<u64>) {
    {
        match screen {
            Screen::Setup => (Pattern::Setup, None),
            Screen::Boot => {
                let done = m
                    .boot
                    .steps
                    .iter()
                    .take_while(|(s, _)| matches!(s, Step::Done | Step::Failed))
                    .count();
                (
                    Pattern::Boot {
                        done: done as u8,
                        total: m.boot.steps.len() as u8,
                    },
                    None,
                )
            }
            Screen::Alert => (Pattern::Alert, None),
            Screen::Night => (Pattern::Night, None),
            Screen::Away | Screen::Sensors => (Pattern::Off, None),
            Screen::Test => (Pattern::Test, None),
            Screen::Rooms => {
                let mut colours = [lightbar::GOOD; 4];
                let mut count = 0;
                let mut pulse = 0u8;
                for (r, geo) in m.rooms.iter().zip(crate::generated::ROOMS.iter()) {
                    if !geo.4 || count == 4 {
                        continue;
                    }
                    colours[count] = state_colour(r.state.unwrap_or(State::Good));
                    if r.state.is_some_and(|s| s.is_alert()) {
                        pulse |= 1 << count;
                    }
                    count += 1;
                }
                (
                    Pattern::Rooms {
                        colours,
                        count: count.max(1) as u8,
                        pulse,
                    },
                    None,
                )
            }
            Screen::Home => {
                if let Some(t) = &m.toast
                    && now.saturating_sub(t.at_ms) < 5500
                {
                    return (Pattern::Confirm, Some(t.at_ms));
                }
                if let Some(g) = greeting_at
                    && now.saturating_sub(g) < GREETING_MS
                {
                    return (Pattern::Greeting, Some(g));
                }
                if m.hub == Hub::Offline {
                    return (Pattern::Offline, None);
                }
                if m.thinking.is_some() {
                    return (Pattern::Thinking, None);
                }
                if let Some(s) = &m.statement
                    && now.saturating_sub(s.at_ms) < unread_ms
                {
                    return (Pattern::Unread, Some(s.at_ms));
                }
                let state = overall(&m.here);
                if state == State::Good {
                    return (Pattern::Calm(lightbar::GOOD), None);
                }
                // air severity meter
                let co2 = m.here.co2.map(crate::air::co2);
                let frac = match co2 {
                    Some(j) if j.state == state => m.here.co2.unwrap_or(0) as f32 / 2000.0,
                    _ => match state {
                        State::Moderate => 0.5,
                        State::Poor => 0.66,
                        _ => 0.9,
                    },
                };
                (Pattern::Meter(frac.min(1.0)), None)
            }
        }
    }
}

fn state_colour(s: State) -> Rgb {
    match s {
        State::Good | State::Neutral => lightbar::GOOD,
        State::Moderate => lightbar::WARN,
        State::Poor => lightbar::POOR,
        State::Bad => lightbar::BAD,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Alert, Metric, Readings, Statement};

    fn calm_model() -> Model {
        let mut m = Model::default();
        for s in m.boot.steps.iter_mut() {
            s.0 = Step::Done;
        }
        m.hub = Hub::Online;
        m.here = Readings {
            temp_c: Some(22.4),
            humidity: Some(41.0),
            co2: Some(642),
            pm2_5: Some(6),
            voc: Some(118),
            lux: Some(320.0),
            presence: true,
        };
        // local evening time
        m.clock = Some((
            crate::clock::parse_iso8601("2026-10-04T19:42:00-04:00").unwrap(),
            0,
            -240,
        ));
        m
    }

    #[test]
    fn boot_then_home() {
        let mut d = Director::new(Settings::default());
        let mut m = calm_model();
        m.boot.steps[4].0 = Step::Running;
        assert_eq!(d.decide(&m, 0).screen, Screen::Boot);
        m.boot.steps[4].0 = Step::Done;
        assert_eq!(d.decide(&m, 100).screen, Screen::Boot); // brief hold time
        assert_eq!(d.decide(&m, 2000).screen, Screen::Home);
        assert_eq!(d.decide(&m, 2000).pattern, Pattern::Calm(lightbar::GOOD));
    }

    #[test]
    fn away_after_ten_minutes_and_greeting_on_return() {
        let mut d = Director::new(Settings::default());
        let mut m = calm_model();
        d.decide(&m, 5000);
        m.here.presence = false;
        assert_eq!(d.decide(&m, 5000 + 9 * 60_000).screen, Screen::Home);
        assert_eq!(d.decide(&m, 5000 + 10 * 60_000).screen, Screen::Away);
        m.here.presence = true;
        let back = d.decide(&m, 5000 + 11 * 60_000);
        assert_eq!(back.screen, Screen::Home);
        assert_eq!(back.pattern, Pattern::Greeting);
    }

    #[test]
    fn alert_wins_and_night_dims() {
        let mut d = Director::new(Settings::default());
        let mut m = calm_model();
        m.clock = Some((
            crate::clock::parse_iso8601("2026-10-04T23:48:00-04:00").unwrap(),
            0,
            -240,
        ));
        assert_eq!(d.decide(&m, 10_000).screen, Screen::Night);
        m.alert = Some(Alert {
            title: crate::model::truncated("Smoke or cooking particles"),
            place: crate::model::truncated("Kitchen"),
            since_ms: 0,
            metric: Metric::Pm2_5,
            value: 87,
            normal: crate::model::truncated("Normal is under 12"),
            message: crate::model::truncated("Probably the stove."),
            done: heapless::Vec::new(),
            ask: None,
        });
        let dd = d.decide(&m, 11_000);
        assert_eq!((dd.screen, dd.pattern), (Screen::Alert, Pattern::Alert));
    }

    #[test]
    fn co2_drives_the_meter_and_statements_breathe() {
        let mut d = Director::new(Settings::default());
        let mut m = calm_model();
        m.here.co2 = Some(1180);
        assert_eq!(d.decide(&m, 10_000).pattern, Pattern::Meter(0.59));
        m.statement = Some(Statement {
            text: crate::model::truncated("hi"),
            at_ms: 10_000,
            ts: None,
            chips: heapless::Vec::new(),
        });
        let dd = d.decide(&m, 11_000);
        assert_eq!(dd.pattern, Pattern::Unread);
        assert!((dd.pattern_t - 1.0).abs() < 1e-3);
    }
}
