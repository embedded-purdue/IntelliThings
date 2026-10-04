#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

#[derive(Debug, Clone, Copy)]
struct SensorResponse([u8; 6]);

#[derive(Debug, Clone, Copy)]
struct FormattedData {
    temperature_c: f32,
    temperature_f: f32,
    humidity_percent: f32,
}

impl SensorResponse {
    fn convert(&self) -> FormattedData {
        let raw_temperature = u16::from_be_bytes([self.0[0], self.0[1]]) as f32;

        let raw_humidity = u16::from_be_bytes([self.0[3], self.0[4]]) as f32;
        
        let temperature_c = -45.0 + 175.0 * (raw_temperature / 65535.0);

        let temperature_f = -49.0 + 315.0 * (raw_temperature / 65535.0);
        
        let humidity_percent = -6.0 + 125.0 * (raw_humidity / 65535.0);
    }
}

use core::fmt;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{
    i2c::master::{Config as I2cConfig, I2c},
    time::Rate,
};
use esp_println::println;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
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

    let i2c_config = I2cConfig::default()
        .with_frequency(Rate::from_khz(100));

    let mut i2c = I2c::new(peripherals.I2C0, i2c_config)
        .expect("Failed to initialize I2C")
        .with_sda(sda)
        .with_scl(scl)
        .into_async();

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let _wifi_controller =
        esp_radio::wifi::WifiController::new(peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");
    let _wifi_interface = esp_radio::wifi::Interface::station();

    // TODO: Spawn some tasks
    let _ = spawner;

    const SHT41_ADDRESS: u8 = 0x44;

    

    loop {
        i2c.write_async(SHT41_ADDRESS, &[0xFD])
        .await
        .expect("Failed to start SHT41 measurement");

        Timer::after(Duration::from_millis(10)).await;

        let mut response = [0u8; 6];

        i2c.read_async(SHT41_ADDRESS, &mut response)
            .await
            .expect("Failed to read SHT41 response");

        println!("SHT41 response: {:02x?}", response);

        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.2.2/examples
}
