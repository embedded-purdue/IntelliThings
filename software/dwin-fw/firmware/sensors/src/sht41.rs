//! sht41 temperature, humidity

use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::I2c;

use crate::{Error, sensirion};

pub const ADDRESS: u8 = 0x44;

const MEASURE_HIGH_PRECISION: u8 = 0xFD;
const SERIAL_NUMBER: u8 = 0x89;

/// one reading
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Climate {
    pub temperature_c: f32,
    pub humidity_pct: f32,
}

#[derive(Debug, Default)]
pub struct Sht41;

impl Sht41 {
    /// 32-bit serial number
    pub async fn serial_number<I: I2c>(
        &self,
        i2c: &mut I,
        delay: &mut impl DelayNs,
    ) -> Result<u32, Error<I::Error>> {
        let words: [u16; 2] = sensirion::query(i2c, delay, ADDRESS, &[SERIAL_NUMBER], 10).await?;
        Ok(sensirion::join(&words) as u32)
    }

    /// high precision, 8.3 ms
    pub async fn measure<I: I2c>(
        &self,
        i2c: &mut I,
        delay: &mut impl DelayNs,
    ) -> Result<Climate, Error<I::Error>> {
        let [t, rh] = sensirion::query(i2c, delay, ADDRESS, &[MEASURE_HIGH_PRECISION], 10).await?;
        Ok(Climate {
            temperature_c: temperature_c(t),
            humidity_pct: humidity_pct(rh),
        })
    }
}

/// datasheet section 4.6
pub fn temperature_c(ticks: u16) -> f32 {
    -45.0 + 175.0 * f32::from(ticks) / 65535.0
}

/// cropped to 0-100 %
pub fn humidity_pct(ticks: u16) -> f32 {
    (-6.0 + 125.0 * f32::from(ticks) / 65535.0).clamp(0.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions_match_the_datasheet() {
        assert!((temperature_c(0x6666) - 25.0).abs() < 0.01);
        assert!((humidity_pct(0x8000) - 56.5).abs() < 0.01);
        assert_eq!(humidity_pct(0), 0.0);
        assert_eq!(humidity_pct(0xFFFF), 100.0);
    }
}
