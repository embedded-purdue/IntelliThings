//! rmt ws2812b driver

use companion_ui::lightbar::Rgb;
use esp_hal::Blocking;
use esp_hal::gpio::Level;
use esp_hal::rmt::{Channel, Error, PulseCode, Tx, TxChannelConfig};
use esp_hal::time::{Duration, Instant};

/// led strip count
pub const LEDS: usize = 24;

/// rmt clock mhz
pub const RMT_MHZ: u32 = 80;
/// ws2812 bit timing
const T0: PulseCode = PulseCode::new(Level::High, 28, Level::Low, 72);
const T1: PulseCode = PulseCode::new(Level::High, 56, Level::Low, 48);

/// idle low config
pub fn channel_config() -> TxChannelConfig {
    TxChannelConfig::default()
        .with_clk_divider(1)
        .with_idle_output(true)
        .with_idle_output_level(Level::Low)
}

/// pixel pulse buffer
pub struct Pixels<'d, const N: usize, const CODES: usize> {
    /// active rmt channel
    channel: Option<Channel<'d, Blocking, Tx>>,
    /// pulse code buffer
    codes: [PulseCode; CODES],
}

/// full led bar
pub type LedBar<'d> = Pixels<'d, LEDS, { LEDS * 24 + 1 }>;
/// onboard rgb led
pub type OnboardLed<'d> = Pixels<'d, 1, 25>;

impl<'d, const N: usize, const CODES: usize> Pixels<'d, N, CODES> {
    pub fn new(channel: Channel<'d, Blocking, Tx>) -> Self {
        assert!(CODES == N * 24 + 1);
        Self {
            channel: Some(channel),
            codes: [PulseCode::end_marker(); CODES],
        }
    }

    /// show pixel frame
    pub fn show(&mut self, frame: &[Rgb; N]) -> Result<(), Error> {
        let Some(channel) = self.channel.take() else {
            return Err(Error::TransmissionError);
        };
        for (i, px) in frame.iter().enumerate() {
            for (k, byte) in [px.g, px.r, px.b].into_iter().enumerate() {
                for bit in 0..8 {
                    self.codes[i * 24 + k * 8 + bit] =
                        if byte & (0x80 >> bit) != 0 { T1 } else { T0 };
                }
            }
        }
        self.codes[N * 24] = PulseCode::end_marker();
        let codes = &self.codes;
        let (result, channel) = critical_section::with(|_| {
            let mut tx = match channel.transmit(codes) {
                Ok(tx) => tx,
                Err((e, channel)) => return (Err(e), Some(channel)),
            };
            let start = Instant::now();
            while !tx.poll() {
                if start.elapsed() > Duration::from_millis(5) {
                    return (Err(Error::TransmissionError), None);
                }
            }
            match tx.wait() {
                Ok(channel) => (Ok(()), Some(channel)),
                Err((e, channel)) => (Err(e), Some(channel)),
            }
        });
        self.channel = channel;
        result
    }
}
