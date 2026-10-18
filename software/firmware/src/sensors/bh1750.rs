use core::fmt;
use embassy_time::{Duration, Timer};
use esp_hal::{i2c::master::I2c, Async};

#[derive(Debug, Default)]
pub struct BH1750 {
    lux: Option<f32>,
}

impl BH1750 {
    pub fn new() -> Self {
        Self { lux: None }
    }

    /// Returns the most recent successful reading, if available.
    pub fn lux(&self) -> Option<f32> {
        self.lux
    }

    /// Takes one high-resolution measurement using the shared I2C bus.
    pub async fn read(
        &mut self,
        i2c: &mut I2c<'_, Async>,
    ) -> Result<(), &'static str> {
        // ADDR low = 0x23; ADDR high = 0x5C.
        const ADDRESS: u8 = 0x23;
        const POWER_ON: u8 = 0x01;
        const ONE_TIME_HIGH_RES: u8 = 0x20;

        // Clear the previous reading so a failure cannot appear as fresh data.
        self.lux = None;

        i2c.write_async(ADDRESS, &[POWER_ON])
            .await
            .map_err(|_| "Failed to power on BH1750")?;

        i2c.write_async(ADDRESS, &[ONE_TIME_HIGH_RES])
            .await
            .map_err(|_| "Failed to start BH1750 measurement")?;

        // Datasheet maximum is 180 ms at the default measurement time.
        Timer::after(Duration::from_millis(180)).await;

        let mut response = [0u8; 2];

        i2c.read_async(ADDRESS, &mut response)
            .await
            .map_err(|_| "Failed to read BH1750 response")?;

        // High byte comes first. Default H-resolution scaling is raw / 1.2.
        let raw = u16::from_be_bytes(response);
        self.lux = Some(f32::from(raw) / 1.2);

        Ok(())
    }
}

impl fmt::Display for BH1750 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.lux {
            Some(lux) => write!(f, "Light: {:.2} lux", lux),
            None => write!(f, "Light: unavailable"),
        }
    }
}