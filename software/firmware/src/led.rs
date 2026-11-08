//! Async WS2812B strip driver for the ESP32-C6.

extern crate alloc;

use alloc::{vec, vec::Vec};
use esp_hal::{
    Async,
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

/// A strip whose pixels all display the same RGB color.
pub struct Led<'d> {
    channel: Channel<'d, Async, Tx>,
    frame: Vec<PulseCode>,
}

impl<'d> Led<'d> {
    /// Initializes RMT channel 0 on the strip's data pin.
    ///
    /// Initialize the firmware heap before calling this constructor. The frame
    /// buffer is allocated once and reused for every color update.
    pub fn new(
        rmt: RMT<'d>,
        pin: impl PeripheralOutput<'d>,
        pixel_count: usize,
    ) -> Result<Self, ConfigError> {
        let rmt = Rmt::new(rmt, Rate::from_mhz(80))?.into_async();
        let config = TxChannelConfig::default()
            .with_clk_divider(2)
            .with_memsize(2)
            .with_idle_output(true)
            .with_idle_output_level(Level::Low);
        let channel = rmt.channel0.configure_tx(&config)?.with_pin(pin);

        let pixel_data_len = pixel_count * BITS_PER_PIXEL;
        let mut frame = vec![ZERO; pixel_data_len + 2];
        frame[pixel_data_len] = RESET;
        frame[pixel_data_len + 1] = PulseCode::end_marker();
        Ok(Self { channel, frame })
    }

    /// Sets every pixel to the given RGB values (0–255).
    pub async fn set_color(&mut self, red: u8, green: u8, blue: u8) -> Result<(), Error> {
        let mut pixel = [ZERO; BITS_PER_PIXEL];
        // WS2812B takes green first, with the most significant bit first.
        for (component, value) in pixel.chunks_exact_mut(8).zip([green, red, blue]) {
            for (bit, pulse) in component.iter_mut().enumerate() {
                *pulse = if value & (0x80 >> bit) != 0 {
                    ONE
                } else {
                    ZERO
                };
            }
        }

        let pixel_data_len = self.frame.len() - 2;
        for pulses in self.frame[..pixel_data_len].chunks_exact_mut(BITS_PER_PIXEL) {
            pulses.copy_from_slice(&pixel);
        }
        self.channel.transmit(&self.frame).await
    }

    /// Turns off every pixel.
    pub async fn off(&mut self) -> Result<(), Error> {
        self.set_color(0, 0, 0).await
    }
}
