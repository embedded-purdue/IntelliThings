//! sensirion i2c framing

use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::I2c;

use crate::Error;

pub(crate) fn crc8(bytes: &[u8]) -> u8 {
    let mut crc = 0xFF_u8;
    for &byte in bytes {
        crc ^= byte;
        for _ in 0..8 {
            crc = if crc & 0x80 != 0 {
                (crc << 1) ^ 0x31
            } else {
                crc << 1
            };
        }
    }
    crc
}

/// command plus args
pub(crate) async fn write<I: I2c>(
    i2c: &mut I,
    address: u8,
    command: &[u8],
    args: &[u16],
) -> Result<(), Error<I::Error>> {
    let mut frame = [0u8; 8];
    frame[..command.len()].copy_from_slice(command);
    let mut len = command.len();
    for &word in args {
        let [hi, lo] = word.to_be_bytes();
        frame[len..len + 3].copy_from_slice(&[hi, lo, crc8(&[hi, lo])]);
        len += 3;
    }
    i2c.write(address, &frame[..len]).await.map_err(Error::I2c)
}

/// read crc-checked words
pub(crate) async fn read<I: I2c, const N: usize>(
    i2c: &mut I,
    address: u8,
) -> Result<[u16; N], Error<I::Error>> {
    let mut raw = [0u8; 9];
    let raw = &mut raw[..3 * N];
    i2c.read(address, raw).await.map_err(Error::I2c)?;
    let mut words = [0u16; N];
    for (word, &[hi, lo, crc]) in words.iter_mut().zip(raw.as_chunks::<3>().0) {
        if crc8(&[hi, lo]) != crc {
            return Err(Error::Crc);
        }
        *word = u16::from_be_bytes([hi, lo]);
    }
    Ok(words)
}

/// command, wait, read
pub(crate) async fn query<I: I2c, const N: usize>(
    i2c: &mut I,
    delay: &mut impl DelayNs,
    address: u8,
    command: &[u8],
    wait_ms: u32,
) -> Result<[u16; N], Error<I::Error>> {
    write(i2c, address, command, &[]).await?;
    delay.delay_ms(wait_ms).await;
    read(i2c, address).await
}

/// join serial words
pub(crate) fn join(words: &[u16]) -> u64 {
    words.iter().fold(0, |acc, &w| (acc << 16) | u64::from(w))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_matches_the_datasheet_example() {
        // datasheet crc example
        assert_eq!(crc8(&[0xBE, 0xEF]), 0x92);
        // sgp40 default args
        assert_eq!(crc8(&[0x80, 0x00]), 0xA2);
        assert_eq!(crc8(&[0x66, 0x66]), 0x93);
    }

    #[test]
    fn serial_words_join_most_significant_first() {
        assert_eq!(join(&[0x1234, 0x5678, 0x9ABC]), 0x1234_5678_9ABC);
    }
}
