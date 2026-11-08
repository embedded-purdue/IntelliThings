#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::cell::RefCell;
use embassy_executor::Spawner;

use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::peripherals::WIFI;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{
    i2c::master::{Config as I2cConfig, I2c},
    time::Rate,
    uart::{Config as UartConfig, UartTx},
};
use esp_println::println;
use intelli_things::display::{self, Readings};
use esp_hal::rmt::PulseCode;
use intelli_things::{
    led::{self, Led},
    mqtt, sensors, sntp, wifi,
};
use static_cell::ConstStaticCell;

#[panic_handler]
fn panic(panic_info: &core::panic::PanicInfo) -> ! {
    println!("Panic! {}", panic_info);
    println!("Restarting ESP32 in 1 second");
    esp_hal::delay::Delay::new().delay_millis(1_000);
    esp_hal::system::software_reset()
}

extern crate alloc;

/// Owns Wi-Fi: connects, then starts everything that needs the network.
/// Its own task, so the sensors and display run while Wi-Fi connects or retries.
#[embassy_executor::task]
async fn network_task(spawner: Spawner, wifi: WIFI<'static>) {
    // The controller must stay alive for the connection to stay up, so this task never returns
    let (_wifi_controller, stack) = wifi::setup(spawner, wifi).await;
    // .expect() is safe here since a panic restarts the device
    spawner.spawn(sntp::task(stack).expect("Failed to prepare SNTP task"));
    spawner.spawn(mqtt::publish_task(stack).expect("Failed to prepare MQTT task"));
    core::future::pending::<()>().await;
}

/// Pixels on the WS2812B strip.
const LED_PIXELS: usize = 160;
/// The strip's pulse frame (~15 KB), in its own static memory rather than the shared heap.
static LED_FRAME: ConstStaticCell<[PulseCode; led::frame_len(LED_PIXELS)]> =
    ConstStaticCell::new([led::BLANK; led::frame_len(LED_PIXELS)]);

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[allow(
    clippy::await_holding_refcell_ref,
    reason = "all bus users run sequentially in this task; each borrow ends before the next sensor call"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.4.0
    // generator parameters: -o esp32c6 -o esp32c6-wroom-1 -o unstable-hal -o alloc -o embassy -o wifi -o zed

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // The following pins are used to bootstrap the chip. They are available
    // for use, but check the datasheet of the module for more information on them.
    // - GPIO4
    // - GPIO5
    // - GPIO8
    // - GPIO9
    // - GPIO15
    // These GPIO pins are in use by some feature of the module and should not be used.
    let _gpio24 = peripherals.GPIO24;
    let _gpio25 = peripherals.GPIO25;
    let _gpio26 = peripherals.GPIO26;
    let _gpio27 = peripherals.GPIO27;
    let _gpio28 = peripherals.GPIO28;
    let _gpio29 = peripherals.GPIO29;
    let _gpio30 = peripherals.GPIO30;

    // ESP32 C6 WROOM 1 SDA AND SCL -> 6 and 7
    let sda = peripherals.GPIO6;
    let scl = peripherals.GPIO7;

    let i2c_config = I2cConfig::default().with_frequency(Rate::from_khz(100));

    let i2c = I2c::new(peripherals.I2C0, i2c_config)
        .expect("Failed to initialize I2C")
        .with_sda(sda)
        .with_scl(scl)
        .into_async();

    // The heap is for Wi-Fi (buffers and scan results) and MQTT. Keep big fixed buffers
    // like the LED frame in statics: when the frame came from here, scans silently
    // dropped access points and Wi-Fi couldn't find the router.
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    // Wi-Fi, then SNTP (display clock) and MQTT
    spawner.spawn(network_task(spawner, peripherals.WIFI).expect("Failed to prepare network task"));

    // DWIN panel on UART0, 115200 8N1 (the default config): node TX (GPIO16) -> panel RX2.
    // GPIO16/17 are also wired to the DevKit's USB-UART bridge, so use the "USB" port for
    // flashing and logs (esp-println logs over USB-JTAG only, so UART0 is free).
    // UART1 belongs to the LD2410C and LP_UART to the PMS5003.
    let display_tx = UartTx::new(peripherals.UART0, UartConfig::default())
        .expect("Failed to initialize display UART")
        .with_tx(peripherals.GPIO16)
        .into_async();
    spawner.spawn(display::task(display_tx).expect("display task"));
    sensors::pms5003::start(
        spawner,
        peripherals.LP_UART,
        peripherals.GPIO4,
        peripherals.GPIO5,
    );

    sensors::ld2410c::start(
        spawner,
        peripherals.UART1,
        peripherals.GPIO20,
        peripherals.GPIO21,
    );

    // All I2C access stays sequential in this task. Release each temporary
    // borrow before calling a driver that uses a RefCellDevice handle.
    let bus = RefCell::new(i2c);
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
    
    let mut led = match Led::new(peripherals.RMT, peripherals.GPIO10, LED_FRAME.take()) {
        Ok(led) => Some(led),
        Err(err) => {
            println!("Failed to initialize LED strip");
            panic!();
        }
    };
    

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

        display::publish(&readings);

        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.2.2/examples
}
