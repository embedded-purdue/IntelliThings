use embassy_time::{Duration, Timer};
use esp_hal::{i2c::master::I2c, Async};
use core::fmt;

/// Raw data from the sensor
type RawResponse = [u8; 6];

#[derive(Debug)]
pub struct SHT41 {
    pub temperature_c: f32,
    pub temperature_f: f32,
    pub humidity_percent: f32,
}

/// Struct representing the SHT41 sensor
impl SHT41 {
    /// Makes a new sensor with all data fields at 0
    pub fn new() -> Self {
        Self { temperature_c: 0.0, temperature_f: 0.0, humidity_percent: 0.0}
    }

    fn convert(&mut self, response: RawResponse) {
        let raw_temperature = u16::from_be_bytes([response[0], response[1]]) as f32;
        let raw_humidity = u16::from_be_bytes([response[3], response[4]]) as f32;

        self.temperature_c = -45.0 + 175.0 * (raw_temperature / 65535.0);
        self.temperature_f = self.temperature_c * 9.0 / 5.0 + 32.0;
        self.humidity_percent = -6.0 + 125.0 * (raw_humidity / 65535.0);
    }

    /// Reads the data from the sensor over I2C and updates the struct
    /// with the converted, human readable values
    /// Returns an error string if the measurement fails
    pub async fn read(&mut self, i2c: &mut I2c<'_, Async>) -> Result<(), &'static str> {
        const ADDRESS: u8 = 0x44;
        i2c.write_async(ADDRESS, &[0xFD])
            .await
            .map_err(|_| "Failed to start SHT41 measurement")?;
    
        Timer::after(Duration::from_millis(10)).await;
    
        let mut response: RawResponse = [0u8; 6];
    
        i2c.read_async(ADDRESS, &mut response)
            .await
            .map_err(|_| "Failed to read SHT41 response")?;
    
        self.convert(response);
        Ok(())
    }
}

impl fmt::Display for SHT41 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Temperature: {:.2} C / {:.2} F, Humidity: {:.2} %RH",
            self.temperature_c,
            self.temperature_f,
            self.humidity_percent
        )
    }
}