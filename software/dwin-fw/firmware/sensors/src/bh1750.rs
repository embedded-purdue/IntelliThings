//! bh1750 ambient light

use embedded_hal_async::i2c::I2c;

use crate::Error;

/// addr pin low
pub const ADDRESS: u8 = 0x23;
/// addr pin high
pub const ADDRESS_HIGH: u8 = 0x5C;

const POWER_ON: u8 = 0x01;
const CONTINUOUS_HIGH_RES: u8 = 0x10;

#[derive(Debug)]
pub struct Bh1750 {
    pub address: u8,
}

impl Default for Bh1750 {
    fn default() -> Self {
        Self { address: ADDRESS }
    }
}

impl Bh1750 {
    /// continuous 1 lx mode
    pub async fn start<I: I2c>(&self, i2c: &mut I) -> Result<(), Error<I::Error>> {
        i2c.write(self.address, &[POWER_ON])
            .await
            .map_err(Error::I2c)?;
        i2c.write(self.address, &[CONTINUOUS_HIGH_RES])
            .await
            .map_err(Error::I2c)
    }

    /// latest result, lux
    pub async fn read<I: I2c>(&self, i2c: &mut I) -> Result<f32, Error<I::Error>> {
        let mut raw = [0u8; 2];
        i2c.read(self.address, &mut raw).await.map_err(Error::I2c)?;
        Ok(lux(u16::from_be_bytes(raw)))
    }
}

/// count / 1.2
pub fn lux(count: u16) -> f32 {
    f32::from(count) / 1.2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_convert_as_the_datasheet_says() {
        // datasheet example values
        assert!((lux(0x8390) - 28066.67).abs() < 0.01);
    }
}
