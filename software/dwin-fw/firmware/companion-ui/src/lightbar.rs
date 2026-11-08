//! lightbar pattern engine

use libm::{cosf, fabsf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn scaled(self, k: f32) -> Self {
        let k = k.clamp(0.0, 1.0);
        let s = |v: u8| (v as f32 * k + 0.5) as u8;
        Self::new(s(self.r), s(self.g), s(self.b))
    }
}

pub const ACCENT: Rgb = Rgb::new(94, 234, 212);
pub const GOOD: Rgb = Rgb::new(74, 222, 128);
pub const WARN: Rgb = Rgb::new(251, 191, 36);
pub const POOR: Rgb = Rgb::new(249, 115, 22);
pub const BAD: Rgb = Rgb::new(239, 68, 68);
pub const WARM: Rgb = Rgb::new(255, 160, 60);
pub const BLUE: Rgb = Rgb::new(80, 140, 255);
pub const NIGHT: Rgb = Rgb::new(255, 110, 30);
pub const WHITE: Rgb = Rgb::new(255, 240, 220);
pub const GREY: Rgb = Rgb::new(40, 40, 40);

/// lightbar display pattern
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pattern {
    Off,
    /// center glow pattern
    Calm(Rgb),
    /// fraction level meter
    Meter(f32),
    /// thinking comet chase
    Thinking,
    /// unread statement breathe
    Unread,
    /// confirmation action sweep
    Confirm,
    /// alert pulse wave
    Alert,
    /// offline edge pixel
    Offline,
    /// night center dim
    Night,
    /// arrival greeting flare
    Greeting,
    /// boot segment progress
    Boot {
        done: u8,
        total: u8,
    },
    /// wifi setup chase
    Setup,
    /// rolling rainbow pattern
    Rainbow,
    /// static test bars
    Test,
    /// room segment colors
    Rooms {
        colours: [Rgb; 4],
        count: u8,
        pulse: u8,
    },
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

/// sine wave fraction
fn wave(t: f32, period: f32) -> f32 {
    0.5 - 0.5 * cosf(t / period * 2.0 * core::f32::consts::PI)
}

/// sample led pixel
pub fn sample(p: &Pattern, t: f32, i: usize, n: usize) -> (Rgb, f32) {
    let fi = i as f32;
    let fnn = n as f32;
    match *p {
        Pattern::Off => (Rgb::default(), 0.0),
        Pattern::Calm(c) => {
            let centre = 1.0 - fabsf(fi - (fnn - 1.0) / 2.0) / (fnn / 2.0);
            (c, 0.06 + 0.10 * wave(t, 6.0) * centre)
        }
        Pattern::Meter(frac) => {
            let x = fi / fnn;
            if x >= frac {
                (GREY, 0.03)
            } else if x < 0.4 {
                (GOOD, 0.75)
            } else if x < 0.6 {
                (WARN, 0.75)
            } else {
                (BAD, 0.75)
            }
        }
        Pattern::Thinking => {
            let head = (t / 2.4) % 1.0 * (fnn + 8.0) - 4.0;
            let d = head - fi;
            if (0.0..6.0).contains(&d) {
                (ACCENT, (1.0 - d / 6.0) * 0.9)
            } else {
                (ACCENT, 0.02)
            }
        }
        Pattern::Unread => (ACCENT, 0.08 + 0.5 * wave(t, 3.0)),
        Pattern::Confirm => {
            if t < 0.5 {
                (WARM, if fi / fnn < t / 0.5 { 0.85 } else { 0.0 })
            } else if t < 5.0 {
                (WARM, 0.85 - (t - 0.5) / 4.5 * 0.55)
            } else {
                (WARM, (0.3 - (t - 5.0) / 0.5 * 0.3).max(0.0))
            }
        }
        Pattern::Alert => {
            let side = 1.0 - fi / (fnn - 1.0).max(1.0);
            (
                BAD,
                clamp01(0.15 + 0.85 * wave(t, 1.0)) * (0.35 + 0.65 * side),
            )
        }
        Pattern::Offline => {
            if i + 1 == n {
                (WARN, 0.15 + 0.6 * wave(t, 2.0))
            } else {
                (WARN, 0.0)
            }
        }
        Pattern::Night => {
            if fnn * 0.35 < fi && fi < fnn * 0.65 {
                (NIGHT, 0.05)
            } else {
                (NIGHT, 0.0)
            }
        }
        Pattern::Greeting => {
            let p = t / 6.0;
            if p >= 0.7 {
                return (WHITE, 0.0);
            }
            let r = p * 1.6 * (fnn / 2.0);
            (
                WHITE,
                clamp01(r - fabsf(fi - (fnn - 1.0) / 2.0)) * 0.55 * (1.0 - p / 0.7 * 0.6),
            )
        }
        Pattern::Boot { done, total } => {
            let total = total.max(1);
            let seg = (fi / fnn * total as f32) as u8;
            if seg < done {
                (GOOD, 0.5)
            } else if seg == done {
                (ACCENT, 0.2 + 0.6 * wave(t, 0.8))
            } else {
                (ACCENT, 0.02)
            }
        }
        Pattern::Setup => {
            let step = (t * 6.0) as usize;
            (
                BLUE,
                if (i + step).is_multiple_of(6) {
                    0.7
                } else {
                    0.05
                },
            )
        }
        Pattern::Rainbow => {
            let h = (fi / fnn + t / 4.0) % 1.0;
            let k = |o: f32| {
                let x = (o + h * 6.0) % 6.0;
                (255.0 * clamp01(x.min(4.0 - x))) as u8
            };
            (Rgb::new(k(5.0), k(3.0), k(1.0)), 0.8)
        }
        Pattern::Test => {
            const BARS: [Rgb; 4] = [
                Rgb::new(255, 0, 0),
                Rgb::new(0, 255, 0),
                Rgb::new(0, 0, 255),
                Rgb::new(255, 255, 255),
            ];
            (BARS[i % 4], 0.6)
        }
        Pattern::Rooms {
            colours,
            count,
            pulse,
        } => {
            let count = count.clamp(1, 4) as usize;
            let seg = (i * count / n.max(1)).min(count - 1);
            // segment divider gap
            if (1..count).any(|k| i == k * n / count) {
                return (colours[seg], 0.0);
            }
            let throb = if pulse & (1 << seg) != 0 {
                0.25 * wave(t, 1.5)
            } else {
                0.0
            };
            (colours[seg], 0.35 + throb)
        }
    }
}

/// render lightbar frame
pub fn render(p: &Pattern, t: f32, cap: f32, out: &mut [Rgb]) {
    let n = out.len();
    for (i, px) in out.iter_mut().enumerate() {
        let (c, a) = sample(p, t, i, n);
        *px = c.scaled(clamp01(a) * clamp01(cap));
    }
}

/// average single pixel
pub fn as_one_pixel(p: &Pattern, t: f32, n: usize, cap: f32) -> Rgb {
    let (mut r, mut g, mut b, mut weight, mut peak) = (0.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32);
    for i in 0..n {
        let (c, a) = sample(p, t, i, n);
        let a = clamp01(a);
        r += c.r as f32 * a;
        g += c.g as f32 * a;
        b += c.b as f32 * a;
        weight += a;
        peak = peak.max(a);
    }
    if weight <= 0.0 {
        return Rgb::default();
    }
    let avg = Rgb::new((r / weight) as u8, (g / weight) as u8, (b / weight) as u8);
    avg.scaled(peak * clamp01(cap))
}

/// estimate frame current
pub fn estimate_ma(frame: &[Rgb]) -> u32 {
    frame
        .iter()
        .map(|c| (c.r as u32 + c.g as u32 + c.b as u32) * 20 / 255)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_is_dark() {
        let mut f = [Rgb::new(1, 2, 3); 24];
        render(&Pattern::Off, 3.0, 1.0, &mut f);
        assert!(f.iter().all(|c| *c == Rgb::default()));
    }

    #[test]
    fn meter_lights_the_requested_fraction() {
        let mut f = [Rgb::default(); 24];
        render(&Pattern::Meter(0.5), 0.0, 1.0, &mut f);
        let lit = f
            .iter()
            .filter(|c| c.r as u32 + c.g as u32 + c.b as u32 > 30)
            .count();
        assert_eq!(lit, 12);
        assert_eq!(f[0], GOOD.scaled(0.75));
        assert_eq!(f[11], WARN.scaled(0.75));
    }

    #[test]
    fn offline_only_lights_the_last_pixel() {
        let mut f = [Rgb::default(); 24];
        render(&Pattern::Offline, 1.0, 1.0, &mut f);
        assert!(f[..23].iter().all(|c| *c == Rgb::default()));
        assert_ne!(f[23], Rgb::default());
    }

    #[test]
    fn cap_limits_current() {
        let mut f = [Rgb::default(); 24];
        render(&Pattern::Rainbow, 0.0, 0.25, &mut f);
        let capped = estimate_ma(&f);
        render(&Pattern::Rainbow, 0.0, 1.0, &mut f);
        assert!(capped * 3 < estimate_ma(&f));
    }

    #[test]
    fn test_pattern_stands_still() {
        let (mut a, mut b) = ([Rgb::default(); 24], [Rgb::default(); 24]);
        render(&Pattern::Test, 0.0, 1.0, &mut a);
        render(&Pattern::Test, 7.3, 1.0, &mut b);
        assert_eq!(a, b);
        assert_eq!(a[0], Rgb::new(153, 0, 0));
        assert_eq!(a[1], Rgb::new(0, 153, 0));
        assert_eq!(a[3], Rgb::new(153, 153, 153));
        assert_eq!(a[4], a[0]);
    }

    #[test]
    fn one_pixel_keeps_colour_and_pulse() {
        assert_eq!(as_one_pixel(&Pattern::Off, 0.0, 24, 1.0), Rgb::default());
        // offline single pixel
        let a = as_one_pixel(&Pattern::Offline, 1.0, 24, 1.0);
        assert!(a.r > 100 && a.g > 70 && a.b < 30, "{a:?}");
        // alert pulse comparison
        let trough = as_one_pixel(&Pattern::Alert, 0.0, 24, 1.0);
        let crest = as_one_pixel(&Pattern::Alert, 0.5, 24, 1.0);
        assert!(crest.r > trough.r * 2);
    }

    #[test]
    fn rooms_have_dark_gaps() {
        let p = Pattern::Rooms {
            colours: [GOOD, WARN, GOOD, GOOD],
            count: 3,
            pulse: 0,
        };
        let mut f = [Rgb::default(); 24];
        render(&p, 0.0, 1.0, &mut f);
        assert_eq!(f[8], Rgb::default());
        assert_eq!(f[16], Rgb::default());
        assert_eq!(f[12], WARN.scaled(0.35));
    }
}
