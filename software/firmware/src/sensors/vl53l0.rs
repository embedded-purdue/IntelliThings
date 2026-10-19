use core::{cell::RefCell, fmt};
use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::{Async, i2c::master::I2c};
use vl53l0x::VL53L0x;

const BUDGET_MICRO_SECONDS: u32 = 33_000;
// Zero selects back-to-back mode, avoiding the library's timed-period byte-order bug.
const PERIOD_MILLISECONDS: u32 = 0;

pub struct VL53L0<'bus, 'hw> {
    sensor: VL53L0x<RefCellDevice<'bus, I2c<'hw, Async>>>,
    distance_mm: Option<u16>,
}

impl<'bus, 'hw> VL53L0<'bus, 'hw> {
    /// Initializes once and starts back-to-back measurements.
    /// The sensor task polls for a result roughly once per second.
    /// Access this bus only sequentially from the same task.
    pub fn new(bus: &'bus RefCell<I2c<'hw, Async>>) -> Result<Self, &'static str> {
        let mut sensor =
            VL53L0x::new(RefCellDevice::new(bus)).map_err(|_| "VL53L0 initialization failed")?;

        let accepted = sensor
            .set_measurement_timing_budget(BUDGET_MICRO_SECONDS)
            .map_err(|_| "I2C communication failed")?;
        if !accepted {
            return Err("VL53L0 timing budget rejected");
        }

        sensor
            .start_continuous(PERIOD_MILLISECONDS)
            .map_err(|_| "VL53L0 start failed")?;

        Ok(Self {
            sensor,
            distance_mm: None,
        })
    }

    /// Returns the distance from the latest successful poll, if available.
    pub fn distance_mm(&self) -> Option<u16> {
        self.distance_mm
    }

    /// Checks once without waiting for a measurement. False means not ready yet.
    /// The library returns raw distance and does not validate ranging quality.
    pub fn read(&mut self) -> Result<bool, &'static str> {
        self.distance_mm = None;
        match self.sensor.read_range_mm() {
            Ok(distance) => {
                self.distance_mm = Some(distance);
                Ok(true)
            }
            Err(nb::Error::WouldBlock) => Ok(false),
            Err(nb::Error::Other(_)) => Err("Failed to read VL53L0 measurement"),
        }
    }
}

impl fmt::Display for VL53L0<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.distance_mm {
            Some(distance) => write!(f, "Distance: {} mm", distance),
            None => write!(f, "Distance: unavailable"),
        }
    }
}
