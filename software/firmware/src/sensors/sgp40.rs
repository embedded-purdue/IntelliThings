use embassy_time::{Duration, Timer};
use esp_hal::{i2c::master::I2c, Async};
use core::fmt;

use gas_index_algorithm::{AlgorithmType, GasIndexAlgorithm};

/// Raw data from the sensor
type RawResponse = [u8; 3];

// Sampling interval: one second
const SAMPLING_INTERVAL: f32 = 1.0;

#[derive(Debug)]
pub struct SGP40 {
    algorithm: GasIndexAlgorithm,
    pub raw_voc: u16,
    pub voc_index: i32
}

/// Struct representing the SHT41 sensor
impl SGP40 {
    /// Makes a new sensor with all data fields at 0
    pub fn new() -> Self {
        Self {
            algorithm: GasIndexAlgorithm::new(AlgorithmType::Voc, SAMPLING_INTERVAL),
            raw_voc: 0,
            voc_index: 0,
        }
    }

    fn convert(&mut self, response: RawResponse) {
        self.raw_voc = u16::from_be_bytes([response[0], response[1]]);
        self.voc_index = self.algorithm.process(i32::from(self.raw_voc));
    }

    /// Reads the data from the sensor over I2C and updates the struct
    /// with the converted, human readable values
    /// Returns an error string if the measurement fails
    pub async fn read(&mut self, i2c: &mut I2c<'_, Async>, temp_c: f32, humidity_percent: f32) -> Result<(), &'static str> {
        const ADDRESS: u8 = 0x59;

        let command = measurement_command(temp_c, humidity_percent);
        
        i2c.write_async(ADDRESS, &command)
            .await
            .map_err(|_| "Failed to start SGP40 measurement")?;
    
        Timer::after(Duration::from_millis(30)).await;
    
        let mut response: RawResponse = [0u8; 3];
    
        i2c.read_async(ADDRESS, &mut response)
            .await
            .map_err(|_| "Failed to read SGP40 response")?;

        if crc8([response[0], response[1]]) != response[2] {
            return Err("SGP40 CRC mismatch");
        }
    
        self.convert(response);
        Ok(())
    }
}

impl fmt::Display for SGP40 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Raw VOC: {}, VOC index: {}",
            self.raw_voc,
            self.voc_index
        )
    }
}

fn crc8(bytes: [u8; 2]) -> u8 {
    let mut crc = 0xFFu8;

    for byte in bytes {
        crc ^= byte;
        for _ in 0..8 {
            crc = if crc & 0x80 != 0 {
                (crc << 1) ^ 0x31
            } else {
                crc << 1
            };
        }
    }

    crc
}

fn measurement_command(temperature_c: f32, humidity_percent: f32) -> [u8; 8] {
    let rh_ticks =
        (humidity_percent.clamp(0.0, 100.0) * 65535.0 / 100.0 + 0.5) as u16;
    let temp_ticks =
        ((temperature_c.clamp(-45.0, 130.0) + 45.0) * 65535.0 / 175.0 + 0.5) as u16;

    let rh = rh_ticks.to_be_bytes();
    let temp = temp_ticks.to_be_bytes();

    [
        0x26, 0x0F,
        rh[0], rh[1], crc8(rh),
        temp[0], temp[1], crc8(temp),
    ]
}