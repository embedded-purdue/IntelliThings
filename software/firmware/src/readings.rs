//! Sensor snapshots shared by the main loop, display, and MQTT publisher.

use core::cell::Cell;
use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};

static LATEST: Mutex<CriticalSectionRawMutex, Cell<Readings>> = Mutex::new(Cell::new(Readings {
    temperature_c: None,
    humidity_percent: None,
    voc_index: None,
    co2_ppm: None,
    lux: None,
    distance_mm: None,
    pm1_0: None,
    pm2_5: None,
    pm10: None,
    presence: None,
}));

/// Replaces the shared snapshot after a complete sensor pass.
pub fn publish(readings: &Readings) {
    LATEST.lock(|latest| latest.set(*readings));
}

/// Copies the snapshot so callers can use it without holding the lock.
pub fn latest() -> Readings {
    LATEST.lock(Cell::get)
}

#[derive(Debug, Default, Clone, Copy, serde::Serialize)]
pub struct Readings {
    pub temperature_c: Option<f32>,

    #[serde(rename = "humidity_pct")]
    pub humidity_percent: Option<f32>,

    pub voc_index: Option<i32>,
    pub co2_ppm: Option<u16>,
    pub lux: Option<f32>,
    pub distance_mm: Option<u16>,

    #[serde(rename = "pm1_0_ugm3")]
    pub pm1_0: Option<u16>,

    #[serde(rename = "pm2_5_ugm3")]
    pub pm2_5: Option<u16>,

    #[serde(rename = "pm10_ugm3")]
    pub pm10: Option<u16>,

    pub presence: Option<bool>,
}

impl Readings {
    /// Writes a complete JSON payload and returns its length in bytes.
    /// Only send `buffer[..len]` after serialization succeeds.
    pub fn to_json(
        &self,
        node_id: &str,
        ts: &str,
        buffer: &mut [u8],
    ) -> Result<usize, serde_json_core::ser::Error> {
        #[derive(serde::Serialize)]
        struct Payload<'a> {
            node_id: &'a str,
            #[serde(flatten)]
            readings: &'a Readings,
            ts: &'a str,
        }

        serde_json_core::to_slice(
            &Payload {
                node_id,
                readings: self,
                ts,
            },
            buffer,
        )
    }
}
