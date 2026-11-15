//! WS2812B strip driver for the ESP32-C6.
//!
//! The pulse frame is a caller-provided buffer (normally a `static`), not a heap
//! allocation, so the strip never takes memory from Wi-Fi's heap.
//!
//! Frames go out with a blocking send while interrupts are off, the same way as
//! `led_strip_test`. The RMT's hardware buffer holds only 96 pulses, so the driver has
//! to keep refilling it during a frame, with about 60 µs to spare each time. esp-hal's
//! async send only refills when the executor gets back to the task, which often comes
//! too late once other tasks are running, so the strip got a broken frame and stayed dark.
//! The cost: interrupts are off for each frame (about 30 µs per pixel plus 0.8 ms of latch).

use embassy_time::Instant;
pub use companion_ui::lightbar::Rgb;
use esp_hal::{
    Blocking,
    gpio::{Level, interconnect::PeripheralOutput},
    peripherals::RMT,
    rmt::{Channel, ConfigError, Error, PulseCode, Rmt, Tx, TxChannelConfig, TxChannelCreator},
    time::Rate,
};

const BITS_PER_PIXEL: usize = 24;
// At 80 MHz with a divider of 2, each tick is 25 ns.
const ZERO: PulseCode = PulseCode::new(Level::High, 16, Level::Low, 34);
const ONE: PulseCode = PulseCode::new(Level::High, 32, Level::Low, 18);
const RESET: PulseCode = PulseCode::new(Level::Low, 16_000, Level::Low, 16_000);

/// Value to fill a new frame buffer with; `Led::new` overwrites it.
pub const BLANK: PulseCode = ZERO;

/// The 24 pulses for one pixel. WS2812B takes green first, most significant bit first.
fn encode(color: Rgb) -> [PulseCode; BITS_PER_PIXEL] {
    let mut pixel = [ZERO; BITS_PER_PIXEL];
    for (component, value) in pixel.chunks_exact_mut(8).zip([color.g, color.r, color.b]) {
        for (bit, pulse) in component.iter_mut().enumerate() {
            if value & (0x80 >> bit) != 0 {
                *pulse = ONE;
            }
        }
    }
    pixel
}

/// Pulses a frame needs for `pixel_count` pixels: 24 per pixel, plus reset and end marker.
pub const fn frame_len(pixel_count: usize) -> usize {
    pixel_count * BITS_PER_PIXEL + 2
}

/// A WS2812B strip on one RMT channel.
pub struct Led<'d> {
    /// `None` after a send that never finished; the strip stays off from then on.
    channel: Option<Channel<'d, Blocking, Tx>>,
    frame: &'d mut [PulseCode],
}

impl<'d> Led<'d> {
    /// Initializes RMT channel 0 on the strip's data pin.
    ///
    /// `frame` holds the pulses for every color update and sets the strip length:
    /// make it `frame_len(pixel_count)` long, e.g. a `static` array filled with `BLANK`.
    pub fn new(
        rmt: RMT<'d>,
        pin: impl PeripheralOutput<'d>,
        frame: &'d mut [PulseCode],
    ) -> Result<Self, ConfigError> {
        assert!(
            frame.len() >= 2 && (frame.len() - 2).is_multiple_of(BITS_PER_PIXEL),
            "LED frame must be frame_len(pixel_count) long"
        );
        let rmt = Rmt::new(rmt, Rate::from_mhz(80))?;
        let config = TxChannelConfig::default()
            .with_clk_divider(2)
            // Both TX memory blocks: twice the refill headroom
            .with_memsize(2)
            .with_idle_output(true)
            .with_idle_output_level(Level::Low);
        let channel = rmt.channel0.configure_tx(&config)?.with_pin(pin);

        let pixel_data_len = frame.len() - 2;
        frame[..pixel_data_len].fill(ZERO);
        frame[pixel_data_len] = RESET;
        frame[pixel_data_len + 1] = PulseCode::end_marker();
        Ok(Self {
            channel: Some(channel),
            frame,
        })
    }

    /// Number of pixels the frame buffer was sized for.
    pub fn pixel_count(&self) -> usize {
        (self.frame.len() - 2) / BITS_PER_PIXEL
    }

    /// Sets every pixel to the given RGB values (0–255).
    pub async fn set_color(&mut self, red: u8, green: u8, blue: u8) -> Result<(), Error> {
        let pixel = encode(Rgb::new(red, green, blue));
        let pixel_data_len = self.frame.len() - 2;
        for pulses in self.frame[..pixel_data_len].chunks_exact_mut(BITS_PER_PIXEL) {
            pulses.copy_from_slice(&pixel);
        }
        self.send()
    }

    /// Sends one color per pixel, first pixel nearest DIN.
    ///
    /// Pixels past the end of `pixels` are turned off; extra entries in `pixels` are ignored.
    pub async fn show(&mut self, pixels: &[Rgb]) -> Result<(), Error> {
        let pixel_data_len = self.frame.len() - 2;
        for (i, pulses) in self.frame[..pixel_data_len]
            .chunks_exact_mut(BITS_PER_PIXEL)
            .enumerate()
        {
            pulses.copy_from_slice(&encode(pixels.get(i).copied().unwrap_or_default()));
        }
        self.send()
    }

    /// Turns off every pixel.
    pub async fn off(&mut self) -> Result<(), Error> {
        self.set_color(0, 0, 0).await
    }

    /// Sends the frame with interrupts off so every refill is on time.
    fn send(&mut self) -> Result<(), Error> {
        let Some(channel) = self.channel.take() else {
            return Err(Error::TransmissionError);
        };
        // A frame takes 30 µs a pixel plus the latch; give up well after that
        // rather than hang with interrupts off.
        let limit_us = (self.pixel_count() as u64 * 30 + 800) * 2 + 1_000;
        let frame = &*self.frame;
        let (result, channel) = critical_section::with(|_| {
            let mut tx = match channel.transmit(frame) {
                Ok(tx) => tx,
                Err((err, channel)) => return (Err(err), Some(channel)),
            };
            let start = Instant::now();
            // poll() also does the buffer refills
            while !tx.poll() {
                if start.elapsed().as_micros() > limit_us {
                    return (Err(Error::TransmissionError), None);
                }
            }
            match tx.wait() {
                Ok(channel) => (Ok(()), Some(channel)),
                Err((err, channel)) => (Err(err), Some(channel)),
            }
        });
        self.channel = channel;
        result
    }
}
