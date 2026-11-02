#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

//! sensor node firmware
// gpio6 sda, gpio7 scl

use core::fmt::{self, Write};

use embassy_executor::Spawner;
use embassy_time::{Delay, Duration, Instant, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{BusTimeout, Config as I2cConfig, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_println::{print, println};
use sensors::Station;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panic! {}", info);
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

/// usb console writer
struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        print!("{s}");
        Ok(())
    }
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    println!();
    println!(
        "IntelliThings sensor node {}, ESP32-C6",
        env!("CARGO_PKG_VERSION")
    );
    println!("I2C: SDA GPIO6, SCL GPIO7, 100 kHz");

    // timeout frees stuck bus
    let config = I2cConfig::default()
        .with_frequency(Rate::from_khz(100))
        .with_timeout(BusTimeout::BusCycles(1000));
    let mut i2c = I2c::new(peripherals.I2C0, config)
        .expect("I2C config")
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7)
        .into_async();

    let mut station = Station::new();
    let _ = station.start(&mut i2c, &mut Delay, &mut Console).await;
    // voc needs 1 s
    let mut next = Instant::now();
    loop {
        next += Duration::from_secs(1);
        Timer::at(next).await;
        let _ = station.tick(&mut i2c, &mut Delay, &mut Console).await;
    }
}
