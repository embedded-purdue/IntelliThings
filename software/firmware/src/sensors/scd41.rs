use core::fmt;
use embassy_time::Delay;
use esp_hal::{i2c::master::I2c, Async};
use scd4x::Scd4xAsync;

#[derive(Debug)]
pub struct SCD41 {
    co2_ppm: u16,
    temperature_c: f32,
    temperature_f: f32,
    humidity_percent: f32,
}

impl SCD41 {
    pub fn new() -> Self {
        Self { co2_ppm: 0, temperature_c: 0.0, temperature_f: 0.0, humidity_percent: 0.0 }
    }

    /// Call once at startup to begin periodic measurements (new reading every ~5 s)
    pub async fn start(&mut self, i2c: &mut I2c<'_, Async>) -> Result<(), &'static str> {
        let mut sensor = Scd4xAsync::new(&mut *i2c, Delay);
        // The sensor may already be measuring from before a reboot, so stop it first
        let _ = sensor.stop_periodic_measurement().await;
        sensor
            .start_periodic_measurement()
            .await
            .map_err(|_| "Failed to start SCD41 periodic measurement")
    }

    /// Reads the latest measurement and updates the struct
    pub async fn read(&mut self, i2c: &mut I2c<'_, Async>) -> Result<(), &'static str> {
        let mut sensor = Scd4xAsync::new(&mut *i2c, Delay);

        // CHECK: method name for the data-ready check (e.g. data_ready_status)
        let ready = sensor
            .data_ready_status()
            .await
            .map_err(|_| "Failed to check SCD41 data-ready status")?;
        if !ready {
            return Err("SCD41 data not ready yet");
        }

        let data = sensor
            .measurement()
            .await
            .map_err(|_| "Failed to read SCD41 measurement")?;

        // CHECK: field names and types on the returned struct
        self.co2_ppm = data.co2;
        self.temperature_c = data.temperature;
        self.temperature_f = self.temperature_c * 9.0 / 5.0 + 32.0;
        self.humidity_percent = data.humidity;
        Ok(())
    }
}

impl fmt::Display for SCD41 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CO2: {} ppm, Temperature: {:.2} C / {:.2} F, Humidity: {:.2} %RH",
            self.co2_ppm, self.temperature_c, self.temperature_f, self.humidity_percent
        )
    }
}