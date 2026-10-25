//! DWIN display driver: writes values to the display's VP addresses over UART.
//! Addresses and encodings follow `docs/interfaces/display-vp-map.md` (v1.0).
//! The display's UI turns the values into screens; this module only sends frames.

use esp_hal::{Async, uart::UartTx};

/// Written to a reading's VP when the sensor is missing or faulted; the tile shows "Waiting".
pub const NO_READING: u16 = 0x8000;

/// VP addresses from the display map.
pub mod vp {
    /// First of the 10 sensor words (temperature .. presence).
    pub const SENSORS: u16 = 0x1000;
    /// Write `INPUTS_RESET_MAGIC` here at boot to reset every input to its power-up value.
    pub const INPUTS_RESET: u16 = 0x103F;
    pub const INPUTS_RESET_MAGIC: u16 = 0x5AA5;
    /// `state` word of the node's boot steps: Sensors, Wi-Fi, MQTT broker, Home Assistant.
    /// The boot page stays up while any of them is pending (0) or running (1).
    pub const BOOT_STEPS: [u16; 4] = [0x1620, 0x1640, 0x1660, 0x1680];
    pub const BOOT_DONE: u16 = 2;
}

/// Largest data payload in one write frame (120 words).
const MAX_DATA_BYTES: usize = 240;

/// One snapshot of this node's readings, in physical units.
/// `None` means no reading and is sent as `NO_READING`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Readings {
    pub temperature_c: Option<f32>,
    pub humidity_percent: Option<f32>,
    pub voc_index: Option<i32>,
    pub co2_ppm: Option<u16>,
    pub lux: Option<f32>,
    pub distance_mm: Option<u16>,
    pub pm1_0: Option<u16>,
    pub pm2_5: Option<u16>,
    pub pm10: Option<u16>,
    pub presence: Option<bool>,
}

impl Readings {
    /// Encodes the readings as the 10 words at `vp::SENSORS`, in map order.
    pub fn to_words(&self) -> [u16; 10] {
        [
            // °C × 10, signed
            self.temperature_c.map_or(NO_READING, |t| round(t * 10.0) as i16 as u16),
            // %RH × 10
            self.humidity_percent.map_or(NO_READING, |h| round(h.clamp(0.0, 100.0) * 10.0) as u16),
            // The gas index algorithm reports 0 while it warms up, so only 1–500 counts as a reading
            self.voc_index
                .filter(|v| (1..=500).contains(v))
                .map_or(NO_READING, |v| v as u16),
            self.co2_ppm.map_or(NO_READING, |c| c.min(0x7FFF)),
            // Whole lux, clamped to 32767 so it can't collide with NO_READING
            self.lux.map_or(NO_READING, |l| round(l.clamp(0.0, 32767.0)) as u16),
            self.distance_mm.map_or(NO_READING, |d| d.min(0x7FFF)),
            self.pm1_0.map_or(NO_READING, |p| p.min(0x7FFF)),
            self.pm2_5.map_or(NO_READING, |p| p.min(0x7FFF)),
            self.pm10.map_or(NO_READING, |p| p.min(0x7FFF)),
            self.presence.map_or(NO_READING, u16::from),
        ]
    }
}

/// Rounds to the nearest whole number (core has no `f32::round` without std).
/// The `as` casts that follow saturate, so out-of-range values can't wrap.
fn round(x: f32) -> f32 {
    if x >= 0.0 { x + 0.5 } else { x - 0.5 }
}

pub struct Display<'d> {
    tx: UartTx<'d, Async>,
}

impl<'d> Display<'d> {
    /// Takes a UART TX already configured for 115200 8N1 (the esp-hal default).
    /// The display's "OK" replies aren't read, so its TX line doesn't need to be wired yet.
    pub fn new(tx: UartTx<'d, Async>) -> Self {
        Self { tx }
    }

    /// Resets every display input to its power-up value. Call once when the node boots.
    pub async fn reset_inputs(&mut self) -> Result<(), &'static str> {
        self.write_words(vp::INPUTS_RESET, &[vp::INPUTS_RESET_MAGIC]).await
    }

    /// Marks all four node boot steps done so the display leaves its boot page.
    /// Wi-Fi and MQTT aren't running yet, so nothing reports them step by step.
    pub async fn finish_boot(&mut self) -> Result<(), &'static str> {
        for step in vp::BOOT_STEPS {
            self.write_words(step, &[vp::BOOT_DONE]).await?;
        }
        Ok(())
    }

    /// Sends all 10 sensor readings in one frame. Call every 0.5–1 s.
    pub async fn write_readings(&mut self, readings: &Readings) -> Result<(), &'static str> {
        self.write_words(vp::SENSORS, &readings.to_words()).await
    }

    /// Writes consecutive 16-bit words starting at `vp`:
    /// `5A A5 <len> 82 <VP hi> <VP lo> <data…>`, high byte first, no CRC.
    pub async fn write_words(&mut self, vp: u16, words: &[u16]) -> Result<(), &'static str> {
        let data_len = words.len() * 2;
        if data_len == 0 || data_len > MAX_DATA_BYTES {
            return Err("Display frame must carry 1-120 words");
        }

        let mut frame = [0u8; 6 + MAX_DATA_BYTES];
        let vp = vp.to_be_bytes();
        frame[..6].copy_from_slice(&[0x5A, 0xA5, (3 + data_len) as u8, 0x82, vp[0], vp[1]]);
        let (data, _) = frame[6..6 + data_len].as_chunks_mut::<2>();
        for (bytes, word) in data.iter_mut().zip(words) {
            *bytes = word.to_be_bytes();
        }

        self.send(&frame[..6 + data_len]).await
    }

    async fn send(&mut self, mut bytes: &[u8]) -> Result<(), &'static str> {
        // write_async only fills the free FIFO space, so loop until the whole frame is queued
        while !bytes.is_empty() {
            let written = self
                .tx
                .write_async(bytes)
                .await
                .map_err(|_| "Failed to write display frame")?;
            bytes = &bytes[written..];
        }
        self.tx
            .flush_async()
            .await
            .map_err(|_| "Failed to flush display frame")
    }
}
