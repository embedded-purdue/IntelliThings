#![no_std]
#![no_main]

//! panel redraw test

use companion_ui::dgus;
use companion_ui::generated::{self, sprite};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::uart::{Config as UartConfig, Uart};
use esp_println::println;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panic! {}", info);
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

fn list(part: &generated::Part, items: &[(u16, u16, u16)]) -> heapless::Vec<u16, 32> {
    let mut v = heapless::Vec::new();
    let _ = v.push(((part.icl as u16) << 8) | 0x07);
    let _ = v.push(items.len() as u16);
    for &(x, y, id) in items {
        let _ = v.extend_from_slice(&[x, y, id - part.first]);
    }
    let _ = v.push(0xFF00);
    v
}

#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 32768);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let uart = Uart::new(
        peripherals.UART1,
        UartConfig::default().with_baudrate(115_200),
    )
    .expect("UART1")
    .with_tx(peripherals.GPIO18)
    .with_rx(peripherals.GPIO19)
    .into_async();
    let (_rx, tx) = uart.split();
    let mut link = dgus::Link::new(tx);

    let skin = &generated::SKIN[0];
    let main1 = &generated::MAIN[0];
    let tile = sprite::TILE_CO2_MODERATE;
    let orb = sprite::BOOT_ORB_IDLE_0;

    // empty other layers
    let mut empties: heapless::Vec<&generated::Part, 8> = heapless::Vec::new();
    for p in generated::MAIN.iter().skip(1).chain(generated::OVER.iter()) {
        let _ = empties.push(p);
    }

    loop {
        for p in generated::SKIN
            .iter()
            .chain(generated::MAIN.iter())
            .chain(empties.iter().copied())
        {
            let _ = link.write_words(p.vp, &list(p, &[])).await;
        }
        let _ = link.set_page(generated::page::TEST).await;
        Timer::after(Duration::from_millis(300)).await;
        let _ = link.set_page(generated::page::BOOT).await;
        Timer::after(Duration::from_millis(300)).await;

        println!("step 1: amber tile bottom-left (SKIN), orb bottom-right (MAIN)");
        let _ = link
            .write_words(skin.vp, &list(skin, &[(60, 300, tile)]))
            .await;
        let _ = link
            .write_words(main1.vp, &list(main1, &[(560, 290, orb)]))
            .await;
        Timer::after(Duration::from_secs(8)).await;

        println!("step 2: only MAIN changes - the orb moves to the middle");
        let _ = link
            .write_words(main1.vp, &list(main1, &[(330, 290, orb)]))
            .await;
        Timer::after(Duration::from_secs(8)).await;

        println!("step 3: MAIN empties - the orb should vanish, the tile should stay");
        let _ = link.write_words(main1.vp, &list(main1, &[])).await;
        Timer::after(Duration::from_secs(8)).await;

        println!("step 4: SKIN empties - the tile should vanish");
        let _ = link.write_words(skin.vp, &list(skin, &[])).await;
        Timer::after(Duration::from_secs(8)).await;
    }
}
