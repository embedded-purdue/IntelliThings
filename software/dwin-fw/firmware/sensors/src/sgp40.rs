//! sgp40 voc sensor

use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::I2c;

use crate::{Error, sensirion};

pub const ADDRESS: u8 = 0x59;

const MEASURE_RAW: [u8; 2] = [0x26, 0x0F];
const SELF_TEST: [u8; 2] = [0x28, 0x0E];
const SERIAL_NUMBER: [u8; 2] = [0x36, 0x82];
const SELF_TEST_PASSED: u16 = 0xD400;

#[derive(Debug, Default)]
pub struct Sgp40;

impl Sgp40 {
    /// 48-bit serial number
    pub async fn serial_number<I: I2c>(
        &self,
        i2c: &mut I,
        delay: &mut impl DelayNs,
    ) -> Result<u64, Error<I::Error>> {
        let words: [u16; 3] = sensirion::query(i2c, delay, ADDRESS, &SERIAL_NUMBER, 1).await?;
        Ok(sensirion::join(&words))
    }

    /// built-in self-test, 320 ms
    pub async fn self_test<I: I2c>(
        &self,
        i2c: &mut I,
        delay: &mut impl DelayNs,
    ) -> Result<(), Error<I::Error>> {
        let [result] = sensirion::query(i2c, delay, ADDRESS, &SELF_TEST, 320).await?;
        if result == SELF_TEST_PASSED {
            Ok(())
        } else {
            Err(Error::Unexpected)
        }
    }

    /// raw sample, 30 ms
    pub async fn measure_raw<I: I2c>(
        &self,
        i2c: &mut I,
        delay: &mut impl DelayNs,
        humidity_pct: f32,
        temperature_c: f32,
    ) -> Result<u16, Error<I::Error>> {
        let args = compensation(humidity_pct, temperature_c);
        sensirion::write(i2c, ADDRESS, &MEASURE_RAW, &args).await?;
        delay.delay_ms(30).await;
        let [sraw] = sensirion::read(i2c, ADDRESS).await?;
        Ok(sraw)
    }
}

/// defaults without sht41
pub const DEFAULT_HUMIDITY_PCT: f32 = 50.0;
pub const DEFAULT_TEMPERATURE_C: f32 = 25.0;

/// humidity, temperature ticks
fn compensation(humidity_pct: f32, temperature_c: f32) -> [u16; 2] {
    let ticks = |fraction: f32| libm::roundf(fraction.clamp(0.0, 1.0) * 65535.0) as u16;
    [
        ticks(humidity_pct / 100.0),
        ticks((temperature_c + 45.0) / 175.0),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_compensation_is_the_datasheets() {
        assert_eq!(
            compensation(DEFAULT_HUMIDITY_PCT, DEFAULT_TEMPERATURE_C),
            [0x8000, 0x6666]
        );
    }

    #[test]
    fn compensation_stays_in_range() {
        assert_eq!(compensation(-3.0, -60.0), [0, 0]);
        assert_eq!(compensation(104.0, 150.0), [0xFFFF, 0xFFFF]);
    }
}
