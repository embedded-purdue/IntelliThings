//! once-a-second readout

use core::fmt::{self, Write};

use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::I2c;

use crate::Error;
use crate::bh1750::{self, Bh1750};
use crate::scd41::{self, Scd41};
use crate::sgp40::{self, Sgp40};
use crate::sht41::{self, Climate, Sht41};
use crate::vl53l0x::{self, Vl53l0x};
use crate::voc_index::VocIndex;

/// retry interval, seconds
pub const RETRY_S: u32 = 10;
/// misses before lost
const MISSES: u8 = 3;
/// co2 stale after, seconds
const CO2_STALE_S: u32 = 15;
/// rows between headers
const HEADER_EVERY: u32 = 30;

/// mqtt snapshot fields
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Snapshot {
    pub temperature_c: Option<f32>,
    pub humidity_pct: Option<f32>,
    pub voc_index: Option<u16>,
    pub co2_ppm: Option<u16>,
    pub lux: Option<f32>,
    pub distance_mm: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Sht41,
    Sgp40,
    Scd41,
    Bh1750,
    Vl53l0x,
}

impl Kind {
    /// sht41 first, compensation
    const ALL: [Kind; 5] = [
        Kind::Sht41,
        Kind::Sgp40,
        Kind::Scd41,
        Kind::Bh1750,
        Kind::Vl53l0x,
    ];

    fn name(self) -> &'static str {
        match self {
            Kind::Sht41 => "SHT41",
            Kind::Sgp40 => "SGP40",
            Kind::Scd41 => "SCD41",
            Kind::Bh1750 => "BH1750",
            Kind::Vl53l0x => "VL53L0X",
        }
    }
}

/// answering or not
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Link {
    Up {
        misses: u8,
    },
    /// down until retry_at
    Down {
        retry_at: u32,
    },
}

/// bring-up details
enum Detail {
    None,
    Serial { serial: u64, digits: usize },
    SelfTested { serial: u64 },
}

impl fmt::Display for Detail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Detail::None => Ok(()),
            Detail::Serial { serial, digits } => write!(f, ", serial {serial:0digits$X}"),
            Detail::SelfTested { serial } => write!(f, ", self-test passed, serial {serial:012X}"),
        }
    }
}

pub struct Station {
    /// seconds since start
    seconds: u32,
    rows: u32,
    links: [Link; 5],
    climate: Option<Climate>,
    voc: VocIndex,
    voc_index: Option<u16>,
    co2_ppm: Option<u16>,
    co2_at: u32,
    bh1750: Bh1750,
    lux: Option<f32>,
    tof: Vl53l0x,
    range_mm: Option<u16>,
}

impl Default for Station {
    fn default() -> Self {
        Self::new()
    }
}

impl Station {
    pub fn new() -> Self {
        Self {
            seconds: 0,
            rows: 0,
            links: [Link::Down { retry_at: 0 }; 5],
            climate: None,
            voc: VocIndex::new(),
            voc_index: None,
            co2_ppm: None,
            co2_at: 0,
            bh1750: Bh1750::default(),
            lux: None,
            tof: Vl53l0x::default(),
            range_mm: None,
        }
    }

    /// scan, bring up sensors
    pub async fn start<I: I2c, D: DelayNs, W: Write>(
        &mut self,
        i2c: &mut I,
        delay: &mut D,
        out: &mut W,
    ) -> fmt::Result {
        scan(i2c, out).await?;
        for kind in Kind::ALL {
            let result = self.bring_up(kind, i2c, delay).await;
            write!(out, "  {:<8} 0x{:02X}  ", kind.name(), self.address(kind))?;
            match result {
                Ok(detail) => {
                    self.links[kind as usize] = Link::Up { misses: 0 };
                    writeln!(out, "ok{detail}")?;
                }
                Err(e) => {
                    self.links[kind as usize] = Link::Down { retry_at: RETRY_S };
                    writeln!(
                        out,
                        "not found ({e}): check its wires. Retrying every {RETRY_S} s."
                    )?;
                }
            }
        }
        writeln!(
            out,
            "CO2 arrives every 5 s. The VOC index starts 45 s in, climbs to about 100 over the"
        )?;
        writeln!(
            out,
            "next minute, then learns the room over about an hour (100 = this room's normal)."
        )?;
        writeln!(
            out,
            "In the table: wait = no reading yet, -- = sensor not answering, far = nothing"
        )?;
        writeln!(out, "within about 1.2 m.")
    }

    /// call once a second
    pub async fn tick<I: I2c, D: DelayNs, W: Write>(
        &mut self,
        i2c: &mut I,
        delay: &mut D,
        out: &mut W,
    ) -> fmt::Result {
        self.seconds += 1;
        let now = self.seconds;
        for kind in Kind::ALL {
            let link = self.links[kind as usize];
            match link {
                Link::Up { misses } => match self.read(kind, i2c, delay).await {
                    Ok(()) => self.links[kind as usize] = Link::Up { misses: 0 },
                    Err(_) if misses + 1 < MISSES => {
                        self.links[kind as usize] = Link::Up { misses: misses + 1 };
                    }
                    Err(e) => {
                        self.links[kind as usize] = Link::Down {
                            retry_at: now + RETRY_S,
                        };
                        self.forget(kind);
                        writeln!(
                            out,
                            "[{}] {} lost ({e}). Retrying every {RETRY_S} s.",
                            Uptime(now),
                            kind.name()
                        )?;
                    }
                },
                Link::Down { retry_at } if now >= retry_at => {
                    match self.bring_up(kind, i2c, delay).await {
                        Ok(detail) => {
                            self.links[kind as usize] = Link::Up { misses: 0 };
                            writeln!(out, "[{}] {} found{detail}", Uptime(now), kind.name())?;
                        }
                        Err(_) => {
                            self.links[kind as usize] = Link::Down {
                                retry_at: now + RETRY_S,
                            }
                        }
                    }
                }
                Link::Down { .. } => {}
            }
        }
        if self.rows.is_multiple_of(HEADER_EVERY) {
            self.write_header(out)?;
        }
        self.rows += 1;
        self.write_row(out)
    }

    /// latest readings
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            temperature_c: self.climate.map(|c| c.temperature_c),
            humidity_pct: self.climate.map(|c| c.humidity_pct),
            voc_index: self.voc_index,
            co2_ppm: self.co2_ppm,
            lux: self.lux,
            distance_mm: self.range_mm.filter(|&mm| mm < vl53l0x::OUT_OF_RANGE_MM),
        }
    }

    fn address(&self, kind: Kind) -> u8 {
        match kind {
            Kind::Sht41 => sht41::ADDRESS,
            Kind::Sgp40 => sgp40::ADDRESS,
            Kind::Scd41 => scd41::ADDRESS,
            Kind::Bh1750 => self.bh1750.address,
            Kind::Vl53l0x => vl53l0x::ADDRESS,
        }
    }

    fn up(&self, kind: Kind) -> bool {
        matches!(self.links[kind as usize], Link::Up { .. })
    }

    async fn bring_up<I: I2c, D: DelayNs>(
        &mut self,
        kind: Kind,
        i2c: &mut I,
        delay: &mut D,
    ) -> Result<Detail, Error<I::Error>> {
        match kind {
            Kind::Sht41 => {
                let serial = Sht41.serial_number(i2c, delay).await?;
                Ok(Detail::Serial {
                    serial: u64::from(serial),
                    digits: 8,
                })
            }
            Kind::Sgp40 => {
                Sgp40.self_test(i2c, delay).await?;
                let serial = Sgp40.serial_number(i2c, delay).await?;
                // restart voc algorithm
                self.voc = VocIndex::new();
                Ok(Detail::SelfTested { serial })
            }
            Kind::Scd41 => {
                Scd41.stop(i2c, delay).await?;
                let serial = Scd41.serial_number(i2c, delay).await?;
                Scd41.start(i2c).await?;
                self.co2_at = self.seconds;
                Ok(Detail::Serial { serial, digits: 12 })
            }
            Kind::Bh1750 => {
                // try both addresses
                self.bh1750.address = bh1750::ADDRESS;
                if self.bh1750.start(i2c).await.is_err() {
                    self.bh1750.address = bh1750::ADDRESS_HIGH;
                    if let Err(e) = self.bh1750.start(i2c).await {
                        self.bh1750.address = bh1750::ADDRESS;
                        return Err(e);
                    }
                }
                Ok(Detail::None)
            }
            Kind::Vl53l0x => {
                self.tof.init(i2c, delay).await?;
                Ok(Detail::None)
            }
        }
    }

    async fn read<I: I2c, D: DelayNs>(
        &mut self,
        kind: Kind,
        i2c: &mut I,
        delay: &mut D,
    ) -> Result<(), Error<I::Error>> {
        match kind {
            Kind::Sht41 => self.climate = Some(Sht41.measure(i2c, delay).await?),
            Kind::Sgp40 => {
                let (humidity, temperature) = match self.climate {
                    Some(c) => (c.humidity_pct, c.temperature_c),
                    None => (sgp40::DEFAULT_HUMIDITY_PCT, sgp40::DEFAULT_TEMPERATURE_C),
                };
                let sraw = Sgp40.measure_raw(i2c, delay, humidity, temperature).await?;
                let index = self.voc.process(sraw);
                self.voc_index = (index > 0).then_some(index);
            }
            Kind::Scd41 => {
                if Scd41.data_ready(i2c, delay).await? {
                    let measurement = Scd41.read(i2c, delay).await?;
                    self.co2_ppm = (measurement.co2_ppm > 0).then_some(measurement.co2_ppm);
                    self.co2_at = self.seconds;
                } else if self.seconds - self.co2_at > CO2_STALE_S {
                    return Err(Error::Timeout);
                }
            }
            Kind::Bh1750 => self.lux = Some(self.bh1750.read(i2c).await?),
            Kind::Vl53l0x => self.range_mm = Some(self.tof.read_range_mm(i2c, delay).await?),
        }
        Ok(())
    }

    fn forget(&mut self, kind: Kind) {
        match kind {
            Kind::Sht41 => self.climate = None,
            Kind::Sgp40 => self.voc_index = None,
            Kind::Scd41 => self.co2_ppm = None,
            Kind::Bh1750 => self.lux = None,
            Kind::Vl53l0x => self.range_mm = None,
        }
    }

    fn write_header<W: Write>(&self, out: &mut W) -> fmt::Result {
        write!(out, "\nsensors:")?;
        for (i, kind) in Kind::ALL.into_iter().enumerate() {
            let state = if self.up(kind) { "ok" } else { "not answering" };
            write!(
                out,
                "{} {} {state}",
                if i == 0 { "" } else { "," },
                kind.name()
            )?;
        }
        writeln!(out)?;
        write_columns(
            out,
            [
                "uptime",
                "temperature_c",
                "humidity_pct",
                "co2_ppm",
                "voc_index",
                "lux",
                "distance_mm",
            ]
            .map(Text::from),
        )
    }

    fn write_row<W: Write>(&self, out: &mut W) -> fmt::Result {
        let snapshot = self.snapshot();
        let cell = |kind: Kind, value: Option<Text>| {
            if !self.up(kind) {
                Text::from("--")
            } else {
                value.unwrap_or(Text::from("wait"))
            }
        };
        let decimal = |v: f32| Text::of(format_args!("{v:.1}"));
        let whole = |v: u16| Text::of(format_args!("{v}"));
        let distance = match self.range_mm {
            Some(mm) if mm >= vl53l0x::OUT_OF_RANGE_MM => Some(Text::from("far")),
            other => other.map(whole),
        };
        write_columns(
            out,
            [
                Text::of(format_args!("{}", Uptime(self.seconds))),
                cell(Kind::Sht41, snapshot.temperature_c.map(decimal)),
                cell(Kind::Sht41, snapshot.humidity_pct.map(decimal)),
                cell(Kind::Scd41, snapshot.co2_ppm.map(whole)),
                cell(Kind::Sgp40, snapshot.voc_index.map(whole)),
                cell(Kind::Bh1750, snapshot.lux.map(decimal)),
                cell(Kind::Vl53l0x, distance),
            ],
        )
    }
}

/// fixed-width columns
fn write_columns<W: Write>(out: &mut W, c: [Text; 7]) -> fmt::Result {
    writeln!(
        out,
        "{:>7}  {:>13}  {:>12}  {:>7}  {:>9}  {:>8}  {:>11}",
        c[0], c[1], c[2], c[3], c[4], c[5], c[6]
    )
}

/// probe every address
async fn scan<I: I2c, W: Write>(i2c: &mut I, out: &mut W) -> fmt::Result {
    write!(out, "I2C scan:")?;
    let mut found = 0;
    for address in 0x08..=0x77 {
        if i2c.write(address, &[]).await.is_ok() {
            let name = match address {
                sht41::ADDRESS => "SHT41",
                sgp40::ADDRESS => "SGP40",
                scd41::ADDRESS => "SCD41",
                bh1750::ADDRESS | bh1750::ADDRESS_HIGH => "BH1750",
                vl53l0x::ADDRESS => "VL53L0X",
                _ => "unknown",
            };
            write!(
                out,
                "{} 0x{address:02X} {name}",
                if found == 0 { "" } else { "," }
            )?;
            found += 1;
        }
    }
    match found {
        0 => writeln!(
            out,
            " nothing answered. Check SDA, SCL, and the sensors' power and ground."
        ),
        1 => writeln!(out, " (1 device)"),
        n => writeln!(out, " ({n} devices)"),
    }
}

/// m:ss or h:mm:ss
struct Uptime(u32);

impl fmt::Display for Uptime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (h, m, s) = (self.0 / 3600, self.0 / 60 % 60, self.0 % 60);
        if h > 0 {
            write!(f, "{h}:{m:02}:{s:02}")
        } else {
            write!(f, "{m}:{s:02}")
        }
    }
}

/// small padded cell
#[derive(Clone, Copy)]
struct Text {
    bytes: [u8; 16],
    len: usize,
}

impl Text {
    fn of(args: fmt::Arguments<'_>) -> Self {
        let mut text = Text {
            bytes: [0; 16],
            len: 0,
        };
        let _ = text.write_fmt(args);
        text
    }
}

impl From<&str> for Text {
    fn from(s: &str) -> Self {
        Text::of(format_args!("{s}"))
    }
}

impl Write for Text {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let n = s.len().min(self.bytes.len() - self.len);
        self.bytes[self.len..self.len + n].copy_from_slice(&s.as_bytes()[..n]);
        self.len += n;
        Ok(())
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("?"))
    }
}
