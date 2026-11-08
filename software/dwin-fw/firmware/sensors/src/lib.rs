//! node sensor drivers

#![cfg_attr(not(test), no_std)]

use core::fmt;

use embedded_hal_async::i2c::{ErrorKind, NoAcknowledgeSource};

pub mod bh1750;
pub mod scd41;
mod sensirion;
pub mod sgp40;
pub mod sht41;
pub mod station;
pub mod vl53l0x;
pub mod voc_index;

pub use station::{Snapshot, Station};

/// sensor errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<E> {
    /// bus transfer failed
    I2c(E),
    /// bad reply checksum
    Crc,
    /// wrong id or self-test
    Unexpected,
    /// never became ready
    Timeout,
}

impl<E: embedded_hal_async::i2c::Error> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Error::I2c(e) => match e.kind() {
                ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address) => {
                    "no answer at its address"
                }
                ErrorKind::NoAcknowledge(_) => "it refused a command",
                ErrorKind::ArbitrationLoss => "bus arbitration lost",
                _ => "I2C bus error",
            },
            Error::Crc => "checksum mismatch",
            Error::Unexpected => "unexpected reply",
            Error::Timeout => "timed out",
        })
    }
}
