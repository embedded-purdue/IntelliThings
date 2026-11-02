//! scd41 co2 sensor

use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::I2c;

use crate::{Error, sensirion};

pub const ADDRESS: u8 = 0x62;

const START_PERIODIC_MEASUREMENT: [u8; 2] = [0x21, 0xB1];
const READ_MEASUREMENT: [u8; 2] = [0xEC, 0x05];
const STOP_PERIODIC_MEASUREMENT: [u8; 2] = [0x3F, 0x86];
const GET_DATA_READY_STATUS: [u8; 2] = [0xE4, 0xB8];
const GET_SERIAL_NUMBER: [u8; 2] = [0x36, 0x82];

/// one measurement
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measurement {
    pub co2_ppm: u16,
    pub temperature_c: f32,
    pub humidity_pct: f32,
}

#[derive(Debug, Default)]
pub struct Scd41;

impl Scd41 {
    /// stop, wait 500 ms
    pub async fn stop<I: I2c>(
        &self,
        i2c: &mut I,
        delay: &mut impl DelayNs,
    ) -> Result<(), Error<I::Error>> {
        sensirion::write(i2c, ADDRESS, &STOP_PERIODIC_MEASUREMENT, &[]).await?;
        delay.delay_ms(500).await;
        Ok(())
    }

    /// serial, only while stopped
    pub async fn serial_number<I: I2c>(
        &self,
        i2c: &mut I,
        delay: &mut impl DelayNs,
    ) -> Result<u64, Error<I::Error>> {
        let words: [u16; 3] = sensirion::query(i2c, delay, ADDRESS, &GET_SERIAL_NUMBER, 1).await?;
        Ok(sensirion::join(&words))
    }

    /// measure every 5 s
    pub async fn start<I: I2c>(&self, i2c: &mut I) -> Result<(), Error<I::Error>> {
        sensirion::write(i2c, ADDRESS, &START_PERIODIC_MEASUREMENT, &[]).await
    }

    /// new measurement waiting
    pub async fn data_ready<I: I2c>(
        &self,
        i2c: &mut I,
        delay: &mut impl DelayNs,
    ) -> Result<bool, Error<I::Error>> {
        let [status] = sensirion::query(i2c, delay, ADDRESS, &GET_DATA_READY_STATUS, 1).await?;
        Ok(status & 0x07FF != 0)
    }

    /// read, clears data-ready
    pub async fn read<I: I2c>(
        &self,
        i2c: &mut I,
        delay: &mut impl DelayNs,
    ) -> Result<Measurement, Error<I::Error>> {
        let [co2, t, rh] = sensirion::query(i2c, delay, ADDRESS, &READ_MEASUREMENT, 1).await?;
        Ok(Measurement {
            co2_ppm: co2,
            temperature_c: -45.0 + 175.0 * f32::from(t) / 65535.0,
            humidity_pct: 100.0 * f32::from(rh) / 65535.0,
        })
    }
}
