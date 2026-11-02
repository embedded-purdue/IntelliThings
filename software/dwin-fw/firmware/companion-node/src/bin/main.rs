#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

//! companion bench firmware

use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU32, Ordering};

use companion_node::ledbar::{self, LEDS, LedBar, OnboardLed, RMT_MHZ};
use companion_node::status::{self, Link, STATUS, Status};
use companion_ui::director::{self, Director, PatternClock, Settings};
use companion_ui::lightbar::{self, Pattern, Rgb};
use companion_ui::model::Model;
use companion_ui::ui::Ui;
use companion_ui::vp::{self, Inputs};
use companion_ui::{demo, dgus, generated};
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer, with_timeout};
use embedded_hal_bus::spi::ExclusiveDevice;
use embedded_io_async::{Read, Write};
use esp_hal::Async;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::rmt::{Rmt, TxChannelCreator};
use esp_hal::spi::Mode as SpiMode;
use esp_hal::spi::master::{Config as SpiConfig, Spi};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::uart::{Config as UartConfig, Uart, UartRx, UartTx};
use esp_hal::usb::usb_serial_jtag::{UsbSerialJtag, UsbSerialJtagRx, UsbSerialJtagTx};
use esp_println::{Printer, println};
use mipidsi::Builder;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ST7789;
use mipidsi::options::{ColorInversion, Orientation, Rotation};
use static_cell::ConstStaticCell;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("Panic! {}", info);
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

/// led bar cap
const LED_CAP: f32 = 0.35;
/// onboard led cap
const ONBOARD_CAP: f32 = 0.25;
/// frame period ms
const FRAME_MS: u64 = 50;

/// light pattern state
static LIGHT: Mutex<CriticalSectionRawMutex, Cell<(Pattern, u64)>> =
    Mutex::new(Cell::new((Pattern::Off, 0)));
/// read reply signal
static REPLY: Signal<CriticalSectionRawMutex, (u16, [u16; 4])> = Signal::new();
static ACKS: AtomicU32 = AtomicU32::new(0);
/// rx byte count
static RX_BYTES: AtomicU32 = AtomicU32::new(0);
static RX_ERRORS: AtomicU32 = AtomicU32::new(0);
/// current baud rate
static BAUD: AtomicU32 = AtomicU32::new(115_200);
/// rx tail buffer
static RX_TAIL: Mutex<CriticalSectionRawMutex, Cell<([u8; 12], usize)>> =
    Mutex::new(Cell::new(([0; 12], 0)));
/// scan baud rates
const BAUDS: [u32; 8] = [
    115_200, 9_600, 921_600, 57_600, 38_400, 19_200, 230_400, 460_800,
];
static UI: ConstStaticCell<Ui> = ConstStaticCell::new(Ui::new());
/// shared vp inputs
static INPUTS: Mutex<CriticalSectionRawMutex, RefCell<Inputs>> =
    Mutex::new(RefCell::new(Inputs::new()));
/// lightbar brightness percent
static LED_LEVEL: AtomicU32 = AtomicU32::new(100);
static LCD_BUFFER: ConstStaticCell<[u8; 1024]> = ConstStaticCell::new([0; 1024]);

fn now_ms() -> u64 {
    Instant::now().as_millis()
}

#[embassy_executor::task]
async fn light_bar(mut bar: LedBar<'static>, mut onboard: OnboardLed<'static>) {
    let mut frame = [Rgb::default(); LEDS];
    let mut reported = false;
    loop {
        let (pattern, start) = LIGHT.lock(|c| c.get());
        let t = now_ms().saturating_sub(start) as f32 / 1000.0;
        let level = LED_LEVEL.load(Ordering::Relaxed) as f32 / 100.0;
        lightbar::render(&pattern, t, LED_CAP * level, &mut frame);
        let one = lightbar::as_one_pixel(&pattern, t, LEDS, ONBOARD_CAP * level);
        let bar_sent = bar.show(&frame);
        let one_sent = onboard.show(&[one]);
        if let (Err(e), false) = (bar_sent.and(one_sent), reported) {
            println!("Light bar: RMT error {:?}", e);
            reported = true;
        }
        Timer::after(Duration::from_millis(33)).await;
    }
}

/// panel rx task
#[embassy_executor::task]
async fn panel_rx(mut rx: UartRx<'static, Async>) {
    let mut buf = [0u8; 256];
    let mut have = 0;
    let mut baud = BAUD.load(Ordering::Relaxed);
    loop {
        let want = BAUD.load(Ordering::Relaxed);
        if want != baud {
            baud = want;
            let _ = rx.apply_config(&UartConfig::default().with_baudrate(baud));
            have = 0;
        }
        match with_timeout(Duration::from_millis(100), rx.read_async(&mut buf[have..])).await {
            Err(_) => continue, // baud check timeout
            Ok(Ok(n)) => {
                RX_BYTES.fetch_add(n as u32, Ordering::Relaxed);
                RX_TAIL.lock(|c| {
                    let (mut tail, mut len) = c.get();
                    for &b in &buf[have..have + n] {
                        if len == tail.len() {
                            tail.copy_within(1.., 0);
                            len -= 1;
                        }
                        tail[len] = b;
                        len += 1;
                    }
                    c.set((tail, len));
                });
                have += n;
            }
            Ok(Err(_)) => {
                RX_ERRORS.fetch_add(1, Ordering::Relaxed);
                have = 0; // framing error reset
                continue;
            }
        }
        while let Some((reply, used)) = dgus::parse_reply(&buf[..have]) {
            match reply {
                dgus::Reply::Ack => {
                    ACKS.fetch_add(1, Ordering::Relaxed);
                }
                dgus::Reply::Data { vp, words } => {
                    let mut w = [0u16; 4];
                    for (i, pair) in words.chunks(2).take(4).enumerate() {
                        w[i] = u16::from_be_bytes([pair[0], pair[1]]);
                    }
                    REPLY.signal((vp, w));
                }
                dgus::Reply::Other(_) => {}
            }
            buf.copy_within(used..have, 0);
            have -= used;
        }
        if have == buf.len() {
            have = 0;
        }
    }
}

/// handle input frame
fn handle(req: &vp::Request, reply: &mut [u8; vp::REPLY_MAX], from: &str) -> usize {
    let (len, result) = INPUTS.lock(|c| c.borrow_mut().handle(req, now_ms(), reply));
    if let (Some(Err(e)), vp::Request::Write { vp, words }) = (result, req) {
        println!(
            "{}: write of {} words at {:#06x} refused ({:?}); the VP map is 0x1000-0x17FF",
            from,
            words.len(),
            vp,
            e
        );
    }
    len
}

/// node uart link
#[embassy_executor::task]
async fn node_link(mut rx: UartRx<'static, Async>, mut tx: UartTx<'static, Async>) {
    let mut parser = vp::Parser::new();
    let mut buf = [0u8; 64];
    let mut reply = [0u8; vp::REPLY_MAX];
    loop {
        match rx.read_async(&mut buf).await {
            Ok(n) => {
                for &b in &buf[..n] {
                    if let Some(req) = parser.push(b) {
                        let len = handle(&req, &mut reply, "node link");
                        let _ = tx.write_all(&reply[..len]).await;
                    }
                }
            }
            Err(_) => parser.reset(), // framing error reset
        }
    }
}

/// usb serial task
#[embassy_executor::task]
async fn usb_link(mut rx: UsbSerialJtagRx<'static, Async>, _tx: UsbSerialJtagTx<'static, Async>) {
    let mut parser = vp::Parser::new();
    let mut buf = [0u8; 64];
    let mut reply = [0u8; vp::REPLY_MAX];
    loop {
        let Ok(n) = rx.read(&mut buf).await;
        for &b in &buf[..n] {
            if let Some(req) = parser.push(b) {
                let len = handle(&req, &mut reply, "USB");
                Printer::write_bytes(&reply[..len]);
            }
        }
    }
}

type Panel = dgus::Link<UartTx<'static, Async>>;

/// query panel register
async fn read_word(link: &mut Panel, vp: u16) -> Option<u16> {
    REPLY.reset();
    link.request_read(vp, 1).await.ok()?;
    match with_timeout(Duration::from_millis(500), REPLY.wait()).await {
        Ok((got, w)) if got == vp => Some(w[0]),
        _ => None,
    }
}

/// panel connection state
#[derive(Clone, Copy, PartialEq, Eq)]
enum PanelState {
    /// disconnected search state
    Searching,
    /// factory firmware state
    Factory { page: u16 },
    /// custom firmware active
    Ours { session: u16 },
}

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    // blank leds early
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(RMT_MHZ)).expect("RMT");
    let config = ledbar::channel_config();
    let mut bar = LedBar::new(
        rmt.channel0
            .configure_tx(&config)
            .expect("RMT channel 0")
            .with_pin(peripherals.GPIO20),
    );
    let mut onboard = OnboardLed::new(
        rmt.channel1
            .configure_tx(&config)
            .expect("RMT channel 1")
            .with_pin(peripherals.GPIO8),
    );
    let _ = bar.show(&[Rgb::default(); LEDS]);
    let _ = onboard.show(&[Rgb::default()]);
    spawner.spawn(light_bar(bar, onboard).expect("light bar task"));

    // optional status lcd
    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default()
            .with_frequency(Rate::from_mhz(40))
            .with_mode(SpiMode::_0),
    )
    .expect("SPI2")
    .with_sck(peripherals.GPIO7)
    .with_mosi(peripherals.GPIO6);
    let cs = Output::new(peripherals.GPIO14, Level::High, OutputConfig::default());
    let dc = Output::new(peripherals.GPIO15, Level::Low, OutputConfig::default());
    let rst = Output::new(peripherals.GPIO21, Level::High, OutputConfig::default());
    let backlight = Output::new(peripherals.GPIO22, Level::High, OutputConfig::default());
    let sd_cs = Output::new(peripherals.GPIO4, Level::High, OutputConfig::default());
    let device = ExclusiveDevice::new_no_delay(spi, cs).expect("LCD SPI device");
    let interface = SpiInterface::new(device, dc, LCD_BUFFER.take());
    match Builder::new(ST7789, interface)
        .display_size(172, 320)
        .display_offset(34, 0)
        .invert_colors(ColorInversion::Inverted)
        .orientation(Orientation::new().rotate(Rotation::Deg90))
        .reset_pin(rst)
        .init(&mut Delay::new())
    {
        Ok(lcd) => spawner.spawn(status::task(lcd, backlight, sd_cs).expect("status screen task")),
        Err(_) => println!("No status screen (LCD init failed)"),
    }

    let uart = Uart::new(
        peripherals.UART1,
        UartConfig::default().with_baudrate(115_200),
    )
    .expect("UART1")
    .with_tx(peripherals.GPIO18)
    .with_rx(peripherals.GPIO19)
    .into_async();
    let (rx, tx) = uart.split();
    spawner.spawn(panel_rx(rx).expect("panel rx task"));
    let mut link = dgus::Link::new(tx);

    let node = Uart::new(
        peripherals.UART0,
        UartConfig::default().with_baudrate(115_200),
    )
    .expect("UART0")
    .with_rx(peripherals.GPIO2)
    .with_tx(peripherals.GPIO3)
    .into_async();
    let (node_rx, node_tx) = node.split();
    spawner.spawn(node_link(node_rx, node_tx).expect("node link task"));
    let (usb_rx, usb_tx) = UsbSerialJtag::new(peripherals.USB_DEVICE)
        .into_async()
        .split();
    spawner.spawn(usb_link(usb_rx, usb_tx).expect("USB link task"));

    let ui = UI.take();
    let mut clock = PatternClock::default();
    let start = now_ms();
    let mut state = PanelState::Searching;
    let mut next_panel_check = start;
    let mut next_hint = start;
    let mut misses = 0u8;
    let mut last_scenario = "";
    let mut bytes_sent = 0u32;
    let mut baud_index = 0usize;
    let mut known_baud: Option<u32> = None;
    let mut version = 0u16;
    println!(
        "IntelliThings companion demo: {} scenes, {} s loop",
        demo::SCENARIOS.len(),
        demo::loop_ms() / 1000
    );
    println!("Looking for the DWIN panel on GPIO18 (TX) / GPIO19 (RX) at 115200 baud...");
    println!(
        "Node link (VP frames, see handoff/VP_MAP.html): GPIO2 RX / GPIO3 TX, 115200 baud, or USB"
    );

    // live display model
    let mut live_model = Model::default();
    let mut director = Director::new(Settings::default());
    let mut was_live = false;
    loop {
        let now = now_ms();
        let live = INPUTS.lock(|c| c.borrow().live());
        if live != was_live {
            was_live = live;
            last_scenario = "";
            println!(
                "{}",
                if live {
                    "Live data from the node - the demo stops"
                } else {
                    "No live data - playing the demo"
                }
            );
        }

        let demo_model;
        let (name, screen, model, pattern, since, demo_t) = if live {
            let forced = INPUTS.lock(|c| {
                let mut inputs = c.borrow_mut();
                inputs.apply(&mut live_model, &mut director.settings, now);
                inputs.screen()
            });
            let d = director.decide(&live_model, now);
            let screen = forced.unwrap_or(d.screen);
            let (pattern, since) = if screen == d.screen {
                (d.pattern, d.pattern_t)
            } else {
                let (p, fixed) = director::pattern_for(
                    screen,
                    &live_model,
                    now,
                    None,
                    director.settings.unread_ms,
                );
                (p, clock.update(p, fixed, now))
            };
            (screen.name(), screen, &live_model, pattern, since, None)
        } else {
            let (name, t) = demo::at(now - start);
            let Some((screen, model)) = demo::scenario(name, t, now) else {
                continue;
            };
            demo_model = model;
            // away arrival greeting
            let greeting = (name == "away")
                .then_some(now - t + 10_500)
                .filter(|g| *g <= now);
            let (p, fixed) = director::pattern_for(screen, &demo_model, now, greeting, 120_000);
            (
                name,
                screen,
                &demo_model,
                p,
                clock.update(p, fixed, now),
                Some(t),
            )
        };
        if name != last_scenario {
            let panel = match state {
                PanelState::Searching => "no panel yet",
                PanelState::Factory { .. } => "panel on factory screens",
                PanelState::Ours { .. } => "drawing on the panel",
            };
            println!(
                "> {} {:15} page {} ({}) - {} writes, {} acks, {} rx errors",
                if live { "live" } else { "demo" },
                name,
                screen.page(),
                panel,
                link.frames,
                ACKS.load(Ordering::Relaxed),
                RX_ERRORS.load(Ordering::Relaxed)
            );
            last_scenario = name;
        }

        // drive lightbar output
        let pattern_start = now - (since * 1000.0) as u64;
        LIGHT.lock(|c| c.set((pattern, pattern_start)));
        let (writes, rejected, level) = INPUTS.lock(|c| {
            let mut inputs = c.borrow_mut();
            let panel = match state {
                PanelState::Searching => 0,
                PanelState::Factory { .. } => 1,
                PanelState::Ours { .. } => 2,
            };
            inputs.set_status(panel, screen, live);
            (inputs.writes, inputs.rejected, inputs.led_brightness())
        });
        LED_LEVEL.store(
            if live { (level * 100.0) as u32 } else { 100 },
            Ordering::Relaxed,
        );

        if let PanelState::Ours { .. } = state {
            match ui.frame(&mut link, screen, model, now).await {
                Ok(stats) => bytes_sent = bytes_sent.wrapping_add(stats.bytes as u32),
                Err(e) => println!("UART: {:?}", e),
            }
        }

        let index = demo::SCENARIOS
            .iter()
            .position(|s| s.0 == name)
            .unwrap_or(0);
        STATUS.lock(|c| {
            c.set(Status {
                link: match state {
                    PanelState::Searching => Link::Searching,
                    PanelState::Factory { .. } => Link::Factory,
                    PanelState::Ours { .. } => Link::Ours,
                },
                scene: name,
                scene_index: index as u8,
                scene_count: demo::SCENARIOS.len() as u8,
                scene_ms: demo_t.unwrap_or(1) as u32,
                scene_len_ms: if live {
                    1
                } else {
                    demo::SCENARIOS[index].1 * 1000
                },
                page: screen.page(),
                bytes: bytes_sent,
                version,
                rx_bytes: RX_BYTES.load(Ordering::Relaxed),
                rx_errors: RX_ERRORS.load(Ordering::Relaxed),
                baud: BAUD.load(Ordering::Relaxed),
                pattern,
                pattern_start,
                live,
                writes,
                rejected,
            })
        });

        // poll panel presence
        if now >= next_panel_check {
            next_panel_check = now
                + if state == PanelState::Searching {
                    1000
                } else {
                    2500
                };
            match state {
                PanelState::Searching => {
                    // cycle scan bauds
                    let baud = known_baud.unwrap_or(BAUDS[baud_index % BAUDS.len()]);
                    baud_index += 1;
                    BAUD.store(baud, Ordering::Relaxed);
                    let _ = link
                        .io
                        .apply_config(&UartConfig::default().with_baudrate(baud));
                    Timer::after(Duration::from_millis(120)).await; // rx task settle
                    let mut found = None;
                    for crc in [false, true] {
                        link.crc = crc;
                        if let Some(v) = read_word(&mut link, dgus::sys::VERSION).await {
                            found = Some(v);
                            break;
                        }
                    }
                    if let Some(v) = found {
                        version = v;
                        known_baud = Some(baud);
                        println!(
                            "DWIN panel answered at {} baud (CRC {}): DGUS version {:#04x}, OS version {:#04x}",
                            baud,
                            if link.crc { "on" } else { "off" },
                            v >> 8,
                            v & 0xFF
                        );
                        misses = 0;
                        state = connect(&mut link, ui).await;
                    } else {
                        let bytes = RX_BYTES.load(Ordering::Relaxed);
                        let errors = RX_ERRORS.load(Ordering::Relaxed);
                        let (tail, len) = RX_TAIL.lock(|c| c.get());
                        println!(
                            "no reply at {} baud - received so far: {} bytes, {} errors, last {:02X?}",
                            baud,
                            bytes,
                            errors,
                            &tail[..len]
                        );
                        if now >= next_hint {
                            next_hint = now + 20_000;
                            println!(
                                "(nothing received at all = the panel isn't sending or doesn't hear us: check GP18 -> pin 5 RX2, the divider -> GP19, and that the display is on its normal screen)"
                            );
                        }
                    }
                }
                PanelState::Factory { page } => {
                    match read_word(&mut link, generated::SIGNATURE_VP).await {
                        None => {
                            misses += 1;
                            println!("factory check: no reply to the signature read");
                        }
                        Some(v) if v == generated::SIGNATURE_VALUE => {
                            state = connect(&mut link, ui).await;
                        }
                        Some(v) => {
                            misses = 0;
                            println!(
                                "factory check: signature VP {:#06x} reads {:#06x}, want {:#06x} (the panel runs other screens: reload the SD card, then power-cycle the panel without it)",
                                generated::SIGNATURE_VP,
                                v,
                                generated::SIGNATURE_VALUE
                            );
                            // hold factory page
                            if page == 0 {
                                let _ = link.set_page(0).await;
                                state = PanelState::Factory { page: 1 };
                            }
                        }
                    }
                }
                PanelState::Ours { session } => {
                    match read_word(&mut link, generated::SESSION_VP).await {
                        None => misses += 1,
                        Some(v) if v != session => {
                            println!("Panel restarted - redrawing");
                            misses = 0;
                            state = connect(&mut link, ui).await;
                        }
                        Some(_) => misses = 0,
                    }
                }
            }
            if misses >= 5 {
                println!("Lost the panel - searching again");
                misses = 0;
                state = PanelState::Searching;
                ui.invalidate();
            }
        }
        Timer::after(Duration::from_millis(FRAME_MS)).await;
    }
}

/// check dwin signature
async fn connect(link: &mut Panel, ui: &mut Ui) -> PanelState {
    if read_word(link, generated::SIGNATURE_VP).await != Some(generated::SIGNATURE_VALUE) {
        println!("The panel doesn't have the companion DWIN_SET yet.");
        println!(
            "Copy build/DWIN_SET to its microSD card (make sd DISK=...) and power-cycle it with the card in."
        );
        println!("Stepping through the factory pages meanwhile.");
        return PanelState::Factory { page: 0 };
    }
    let session = 0x5000 | (now_ms() as u16 & 0x0FFF);
    let _ = link.write_words(generated::SESSION_VP, &[session]).await;
    ui.invalidate();
    println!("Companion DWIN_SET found - drawing the demo");
    PanelState::Ours { session }
}
