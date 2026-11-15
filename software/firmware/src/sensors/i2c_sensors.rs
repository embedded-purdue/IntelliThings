use core::cell::RefCell;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use esp_hal::{Async, i2c::master::{Config, I2c}, peripherals::{GPIO6, GPIO7, I2C0}, time::Rate};

use esp_println::println;
use crate::{display, sensors, readings::{self, Readings}};

pub fn start(spawner: Spawner, 
    sda: GPIO6<'static>, 
    scl: GPIO7<'static>,
    i2c0: I2C0<'static>) 
{
    let i2c_config = Config::default().with_frequency(Rate::from_khz(100));
    
    let i2c = I2c::new(i2c0, i2c_config)
        .expect("Failed to initialize I2C")
        .with_sda(sda)
        .with_scl(scl)
        .into_async();
    
    let bus = RefCell::new(i2c);

    spawner.spawn(run(bus).expect("Failed to create I2C task"));
}

#[embassy_executor::task]
async fn run(bus: RefCell<I2c<'static, Async>>) {
    // A missing VL53L0 only blanks the distance tile; panicking would reboot-loop the whole node
    let mut vl53l0 = match sensors::VL53L0::new(&bus) {
        Ok(sensor) => Some(sensor),
        Err(err) => {
            println!("VL53L0 start error: {:?}; continuing without it", err);
            None
        }
    };
    
    let mut sht41 = sensors::SHT41::new();
    let mut bh1750 = sensors::BH1750::new();
    let mut sgp40 = sensors::SGP40::new();
    let mut scd41 = sensors::SCD41::new();
    
    let mut scd41_started = false;
    // Every field starts as "no reading" until its sensor succeeds
    let mut readings = Readings::default();
    
    let start_result = {
        let mut i2c = bus.borrow_mut();
        scd41.start(&mut i2c).await
    };
    match start_result {
        Ok(()) => scd41_started = true,
        Err(err) => println!("SCD41 start error: {}", err),
    }
    
    
    loop {
        let result = {
            let mut i2c = bus.borrow_mut();
            sht41.read(&mut i2c).await
        };
        match result {
            Ok(()) => {
                println!("{}", sht41);
                readings.temperature_c = Some(sht41.temperature_c);
                readings.humidity_percent = Some(sht41.humidity_percent);
            }
            Err(err) => {
                println!("SHT41 error: {}", err);
                readings.temperature_c = None;
                readings.humidity_percent = None;
            }
        }
    
        let result = {
            let mut i2c = bus.borrow_mut();
            bh1750.read(&mut i2c).await
        };
        match result {
            Ok(()) => println!("{}", bh1750),
            Err(err) => println!("BH1750 error: {}", err),
        }
        readings.lux = bh1750.lux();
    
        let result = {
            let mut i2c = bus.borrow_mut();
            sgp40
                .read(&mut i2c, sht41.temperature_c, sht41.humidity_percent)
                .await
        };
        match result {
            Ok(()) => {
                println!("{}", sgp40);
                readings.voc_index = Some(sgp40.voc_index);
            }
            Err(err) => {
                println!("SGP40 error: {}", err);
                readings.voc_index = None;
            }
        }
    
        if scd41_started {
            let result = {
                let mut i2c = bus.borrow_mut();
                scd41.read(&mut i2c).await
            };
            match result {
                Ok(true) => {
                    println!("New SCD41 data: {}", scd41);
                    readings.co2_ppm = Some(scd41.co2_ppm());
                }
                Ok(false) => println!("Old SCD41 data: {}", scd41), // no new reading yet, which is normal
                Err(err) => {
                    println!("SCD41 error: {}", err);
                    readings.co2_ppm = None;
                }
            }
        } else {
            let start_result = {
                let mut i2c = bus.borrow_mut();
                scd41.start(&mut i2c).await
            };
            match start_result {
                Ok(()) => scd41_started = true,
                Err(err) => println!("SCD41 start error: {}", err),
            }
        }
    
        println!("VL53L0:");
        if let Some(vl53l0) = vl53l0.as_mut() {
            match vl53l0.read() {
                Ok(true) => {
                    println!("{}", vl53l0);
                    readings.distance_mm = vl53l0.distance_mm();
                }
                Ok(false) => println!("Measurement still running"), // keep the last distance
                Err(err) => {
                    println!("VL53L0 error: {}", err);
                    readings.distance_mm = None;
                }
            }
        }
    
        // The UART sensors run in their own tasks; take their newest readings.
        // Each returns None when its sensor has gone quiet, so the tile shows "Waiting".
        let pm = sensors::pms5003::latest();
        readings.pm1_0 = pm.map(|pm| pm.pm1_0);
        readings.pm2_5 = pm.map(|pm| pm.pm2_5);
        readings.pm10 = pm.map(|pm| pm.pm10);
        readings.presence = sensors::ld2410c::presence();
    
        readings::publish(&readings);
        display::publish(&readings);
    
        Timer::after(Duration::from_secs(1)).await;
    }
}