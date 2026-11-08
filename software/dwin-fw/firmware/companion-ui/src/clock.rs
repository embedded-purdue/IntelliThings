//! wall clock formatting

use core::fmt::Write;

use heapless::String;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTime {
    pub year: i32,
    pub month: u8,   // 1..=12
    pub day: u8,     // 1..=31
    pub weekday: u8, // 0 is sunday
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

impl DateTime {
    /// local time conversion
    pub fn from_unix(unix: i64, utc_offset_min: i32) -> Self {
        let local = unix + utc_offset_min as i64 * 60;
        let days = local.div_euclid(86_400);
        let secs = local.rem_euclid(86_400);
        let (year, month, day) = civil_from_days(days);
        Self {
            year,
            month,
            day,
            weekday: (days + 4).rem_euclid(7) as u8, // epoch weekday base
            hour: (secs / 3600) as u8,
            minute: (secs / 60 % 60) as u8,
            second: (secs % 60) as u8,
        }
    }

    /// format time string
    pub fn hhmm(&self, h24: bool) -> String<8> {
        let mut s = String::new();
        let h = if h24 {
            self.hour
        } else {
            (self.hour + 11) % 12 + 1
        };
        if h24 {
            let _ = write!(s, "{:02}:{:02}", h, self.minute);
        } else {
            let _ = write!(s, "{}:{:02}", h, self.minute);
        }
        s
    }

    pub fn ampm(&self) -> &'static str {
        if self.hour < 12 { "AM" } else { "PM" }
    }

    /// format date string
    pub fn date_line(&self) -> String<32> {
        let mut s = String::new();
        let _ = write!(
            s,
            "{}, {} {}",
            WEEKDAYS[self.weekday as usize % 7],
            MONTHS[(self.month as usize + 11) % 12],
            self.day
        );
        s
    }
}

/// days to civil
fn civil_from_days(z: i64) -> (i32, u8, u8) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
    ((y + if m <= 2 { 1 } else { 0 }) as i32, m, d)
}

/// parse iso8601 timestamp
pub fn parse_iso8601(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() < 19 {
        return None;
    }
    let num = |from: usize, len: usize| -> Option<i64> {
        let mut v = 0i64;
        for &c in b.get(from..from + len)? {
            if !c.is_ascii_digit() {
                return None;
            }
            v = v * 10 + (c - b'0') as i64;
        }
        Some(v)
    };
    let (y, mo, d) = (num(0, 4)?, num(5, 2)?, num(8, 2)?);
    let (h, mi, sec) = (num(11, 2)?, num(14, 2)?, num(17, 2)?);
    // skip fractional seconds
    let mut i = 19;
    if b.get(i) == Some(&b'.') {
        i += 1;
        while b.get(i).is_some_and(|c| c.is_ascii_digit()) {
            i += 1;
        }
    }
    let offset = match b.get(i) {
        Some(b'Z') | None => 0,
        Some(&sign @ (b'+' | b'-')) => {
            let oh = num(i + 1, 2)?;
            let om = if b.get(i + 3) == Some(&b':') {
                num(i + 4, 2)?
            } else {
                num(i + 3, 2)?
            };
            let o = oh * 3600 + om * 60;
            if sign == b'-' { -o } else { o }
        }
        _ => return None,
    };
    Some(days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + sec - offset)
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// format relative age
pub fn age(seconds: i64) -> String<16> {
    let mut s = String::new();
    let _ = match seconds {
        i64::MIN..60 => write!(s, "just now"),
        60..3600 => write!(s, "{} min ago", seconds / 60),
        3600..86_400 => write!(s, "{} h ago", seconds / 3600),
        _ => write!(s, "{} d ago", seconds / 86_400),
    };
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concept_clock() {
        // test date timestamp
        let unix = parse_iso8601("2026-10-04T19:42:00-04:00").unwrap();
        let t = DateTime::from_unix(unix, -240);
        assert_eq!(t.hhmm(false).as_str(), "7:42");
        assert_eq!(t.ampm(), "PM");
        assert_eq!(t.date_line().as_str(), "Sunday, October 4");
        assert_eq!(t.hhmm(true).as_str(), "19:42");
    }

    #[test]
    fn midnight_and_noon() {
        let t = DateTime::from_unix(parse_iso8601("2026-01-01T00:05:00Z").unwrap(), 0);
        assert_eq!((t.hhmm(false).as_str(), t.ampm()), ("12:05", "AM"));
        let t = DateTime::from_unix(parse_iso8601("2026-01-01T12:00:00Z").unwrap(), 0);
        assert_eq!((t.hhmm(false).as_str(), t.ampm()), ("12:00", "PM"));
        assert_eq!(t.date_line().as_str(), "Thursday, January 1");
    }

    #[test]
    fn iso_round_trip() {
        assert_eq!(parse_iso8601("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(
            parse_iso8601("2026-10-18T14:05:00-04:00"),
            parse_iso8601("2026-10-18T18:05:00Z")
        );
        assert_eq!(
            parse_iso8601("2026-10-18T14:05:00.123+0530"),
            parse_iso8601("2026-10-18T08:35:00Z")
        );
        assert_eq!(parse_iso8601("nope"), None);
    }

    #[test]
    fn ages() {
        assert_eq!(age(5).as_str(), "just now");
        assert_eq!(age(150).as_str(), "2 min ago");
        assert_eq!(age(7200).as_str(), "2 h ago");
    }
}
