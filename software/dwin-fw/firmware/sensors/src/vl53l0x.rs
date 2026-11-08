//! vl53l0x distance sensor
// mit, bsd-3: see licenses.md

use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::I2c;

use crate::Error;

pub const ADDRESS: u8 = 0x29;

/// nothing in range
pub const OUT_OF_RANGE_MM: u16 = 8190;

const SYSRANGE_START: u8 = 0x00;
const SYSTEM_SEQUENCE_CONFIG: u8 = 0x01;
const SYSTEM_INTERRUPT_CONFIG_GPIO: u8 = 0x0A;
const SYSTEM_INTERRUPT_CLEAR: u8 = 0x0B;
const RESULT_INTERRUPT_STATUS: u8 = 0x13;
const RESULT_RANGE_STATUS: u8 = 0x14;
const FINAL_RANGE_CONFIG_MIN_COUNT_RATE_RTN_LIMIT: u8 = 0x44;
const MSRC_CONFIG_TIMEOUT_MACROP: u8 = 0x46;
const DYNAMIC_SPAD_NUM_REQUESTED_REF_SPAD: u8 = 0x4E;
const DYNAMIC_SPAD_REF_EN_START_OFFSET: u8 = 0x4F;
const PRE_RANGE_CONFIG_VCSEL_PERIOD: u8 = 0x50;
const PRE_RANGE_CONFIG_TIMEOUT_MACROP_HI: u8 = 0x51;
const MSRC_CONFIG_CONTROL: u8 = 0x60;
const FINAL_RANGE_CONFIG_VCSEL_PERIOD: u8 = 0x70;
const FINAL_RANGE_CONFIG_TIMEOUT_MACROP_HI: u8 = 0x71;
const GPIO_HV_MUX_ACTIVE_HIGH: u8 = 0x84;
const VHV_CONFIG_PAD_SCL_SDA_EXTSUP_HV: u8 = 0x89;
const GLOBAL_CONFIG_SPAD_ENABLES_REF_0: u8 = 0xB0;
const GLOBAL_CONFIG_REF_EN_START_SELECT: u8 = 0xB6;
const IDENTIFICATION_MODEL_ID: u8 = 0xC0;

/// st default tuning
#[rustfmt::skip]
const TUNING: [(u8, u8); 80] = [
    (0xFF, 0x01), (0x00, 0x00),
    (0xFF, 0x00), (0x09, 0x00), (0x10, 0x00), (0x11, 0x00),
    (0x24, 0x01), (0x25, 0xFF), (0x75, 0x00),
    (0xFF, 0x01), (0x4E, 0x2C), (0x48, 0x00), (0x30, 0x20),
    (0xFF, 0x00), (0x30, 0x09), (0x54, 0x00), (0x31, 0x04), (0x32, 0x03), (0x40, 0x83),
    (0x46, 0x25), (0x60, 0x00), (0x27, 0x00), (0x50, 0x06), (0x51, 0x00), (0x52, 0x96),
    (0x56, 0x08), (0x57, 0x30), (0x61, 0x00), (0x62, 0x00), (0x64, 0x00), (0x65, 0x00),
    (0x66, 0xA0),
    (0xFF, 0x01), (0x22, 0x32), (0x47, 0x14), (0x49, 0xFF), (0x4A, 0x00),
    (0xFF, 0x00), (0x7A, 0x0A), (0x7B, 0x00), (0x78, 0x21),
    (0xFF, 0x01), (0x23, 0x34), (0x42, 0x00), (0x44, 0xFF), (0x45, 0x26), (0x46, 0x05),
    (0x40, 0x40), (0x0E, 0x06), (0x20, 0x1A), (0x43, 0x40),
    (0xFF, 0x00), (0x34, 0x03), (0x35, 0x44),
    (0xFF, 0x01), (0x31, 0x04), (0x4B, 0x09), (0x4C, 0x05), (0x4D, 0x04),
    (0xFF, 0x00), (0x44, 0x00), (0x45, 0x20), (0x47, 0x08), (0x48, 0x28), (0x67, 0x00),
    (0x70, 0x04), (0x71, 0x01), (0x72, 0xFE), (0x76, 0x00), (0x77, 0x00),
    (0xFF, 0x01), (0x0D, 0x01),
    (0xFF, 0x00), (0x80, 0x01), (0x01, 0xF8),
    (0xFF, 0x01), (0x8E, 0x01), (0x00, 0x01), (0xFF, 0x00), (0x80, 0x00),
];

/// busy poll limit
const POLLS: u32 = 250;

// timing overheads, us
const START_OVERHEAD: u32 = 1910;
const END_OVERHEAD: u32 = 960;
const MSRC_OVERHEAD: u32 = 660;
const TCC_OVERHEAD: u32 = 590;
const DSS_OVERHEAD: u32 = 690;
const PRE_RANGE_OVERHEAD: u32 = 660;
const FINAL_RANGE_OVERHEAD: u32 = 550;

#[derive(Debug, Default)]
pub struct Vl53l0x {
    /// st stop variable
    stop_variable: u8,
}

struct Enables {
    tcc: bool,
    dss: bool,
    msrc: bool,
    pre_range: bool,
    final_range: bool,
}

struct Timeouts {
    final_range_vcsel_period_pclks: u8,
    pre_range_mclks: u16,
    msrc_dss_tcc_us: u32,
    pre_range_us: u32,
    final_range_us: u32,
}

impl Vl53l0x {
    /// pololu init, 2v8 i/o
    pub async fn init<I: I2c>(
        &mut self,
        i2c: &mut I,
        delay: &mut impl DelayNs,
    ) -> Result<(), Error<I::Error>> {
        if read(i2c, IDENTIFICATION_MODEL_ID).await? != 0xEE {
            return Err(Error::Unexpected);
        }

        // data init
        let pad = read(i2c, VHV_CONFIG_PAD_SCL_SDA_EXTSUP_HV).await?;
        write(i2c, VHV_CONFIG_PAD_SCL_SDA_EXTSUP_HV, pad | 0x01).await?;
        write_all(
            i2c,
            &[(0x88, 0x00), (0x80, 0x01), (0xFF, 0x01), (0x00, 0x00)],
        )
        .await?;
        self.stop_variable = read(i2c, 0x91).await?;
        write_all(i2c, &[(0x00, 0x01), (0xFF, 0x00), (0x80, 0x00)]).await?;
        // disable rate checks
        let msrc = read(i2c, MSRC_CONFIG_CONTROL).await?;
        write(i2c, MSRC_CONFIG_CONTROL, msrc | 0x12).await?;
        // 0.25 mcps limit
        write16(i2c, FINAL_RANGE_CONFIG_MIN_COUNT_RATE_RTN_LIMIT, 32).await?;
        write(i2c, SYSTEM_SEQUENCE_CONFIG, 0xFF).await?;

        // reference spads
        let (spad_count, spad_type_is_aperture) = self.spad_info(i2c, delay).await?;
        let mut spad_map = [0u8; 6];
        read_into(i2c, GLOBAL_CONFIG_SPAD_ENABLES_REF_0, &mut spad_map).await?;
        write_all(
            i2c,
            &[
                (0xFF, 0x01),
                (DYNAMIC_SPAD_REF_EN_START_OFFSET, 0x00),
                (DYNAMIC_SPAD_NUM_REQUESTED_REF_SPAD, 0x2C),
                (0xFF, 0x00),
                (GLOBAL_CONFIG_REF_EN_START_SELECT, 0xB4),
            ],
        )
        .await?;
        let first_spad_to_enable = if spad_type_is_aperture { 12 } else { 0 };
        let mut spads_enabled = 0;
        for i in 0..48 {
            if i < first_spad_to_enable || spads_enabled == spad_count {
                spad_map[i / 8] &= !(1 << (i % 8));
            } else if (spad_map[i / 8] >> (i % 8)) & 0x1 != 0 {
                spads_enabled += 1;
            }
        }
        let mut frame = [0u8; 7];
        frame[0] = GLOBAL_CONFIG_SPAD_ENABLES_REF_0;
        frame[1..].copy_from_slice(&spad_map);
        i2c.write(ADDRESS, &frame).await.map_err(Error::I2c)?;

        write_all(i2c, &TUNING).await?;

        // interrupt, active low
        write(i2c, SYSTEM_INTERRUPT_CONFIG_GPIO, 0x04).await?;
        let mux = read(i2c, GPIO_HV_MUX_ACTIVE_HIGH).await?;
        write(i2c, GPIO_HV_MUX_ACTIVE_HIGH, mux & !0x10).await?;
        write(i2c, SYSTEM_INTERRUPT_CLEAR, 0x01).await?;

        // drop msrc and tcc
        let budget_us = self.timing_budget_us(i2c).await?;
        write(i2c, SYSTEM_SEQUENCE_CONFIG, 0xE8).await?;
        self.set_timing_budget_us(i2c, budget_us).await?;

        // vhv, phase calibration
        write(i2c, SYSTEM_SEQUENCE_CONFIG, 0x01).await?;
        self.single_ref_calibration(i2c, delay, 0x40).await?;
        write(i2c, SYSTEM_SEQUENCE_CONFIG, 0x02).await?;
        self.single_ref_calibration(i2c, delay, 0x00).await?;
        write(i2c, SYSTEM_SEQUENCE_CONFIG, 0xE8).await
    }

    /// single range, ~33 ms
    pub async fn read_range_mm<I: I2c>(
        &mut self,
        i2c: &mut I,
        delay: &mut impl DelayNs,
    ) -> Result<u16, Error<I::Error>> {
        write_all(
            i2c,
            &[
                (0x80, 0x01),
                (0xFF, 0x01),
                (0x00, 0x00),
                (0x91, self.stop_variable),
                (0x00, 0x01),
                (0xFF, 0x00),
                (0x80, 0x00),
                (SYSRANGE_START, 0x01),
            ],
        )
        .await?;
        poll(i2c, delay, SYSRANGE_START, |v| v & 0x01 == 0).await?;
        poll(i2c, delay, RESULT_INTERRUPT_STATUS, |v| v & 0x07 != 0).await?;
        let range = read16(i2c, RESULT_RANGE_STATUS + 10).await?;
        write(i2c, SYSTEM_INTERRUPT_CLEAR, 0x01).await?;
        Ok(range)
    }

    /// reference spad info
    async fn spad_info<I: I2c>(
        &mut self,
        i2c: &mut I,
        delay: &mut impl DelayNs,
    ) -> Result<(u8, bool), Error<I::Error>> {
        write_all(
            i2c,
            &[(0x80, 0x01), (0xFF, 0x01), (0x00, 0x00), (0xFF, 0x06)],
        )
        .await?;
        let v = read(i2c, 0x83).await?;
        write(i2c, 0x83, v | 0x04).await?;
        write_all(
            i2c,
            &[
                (0xFF, 0x07),
                (0x81, 0x01),
                (0x80, 0x01),
                (0x94, 0x6B),
                (0x83, 0x00),
            ],
        )
        .await?;
        poll(i2c, delay, 0x83, |v| v != 0x00).await?;
        write(i2c, 0x83, 0x01).await?;
        let tmp = read(i2c, 0x92).await?;
        write_all(i2c, &[(0x81, 0x00), (0xFF, 0x06)]).await?;
        let v = read(i2c, 0x83).await?;
        write(i2c, 0x83, v & !0x04).await?;
        write_all(
            i2c,
            &[(0xFF, 0x01), (0x00, 0x01), (0xFF, 0x00), (0x80, 0x00)],
        )
        .await?;
        Ok((tmp & 0x7F, (tmp >> 7) & 0x01 != 0))
    }

    /// one ref calibration
    async fn single_ref_calibration<I: I2c>(
        &mut self,
        i2c: &mut I,
        delay: &mut impl DelayNs,
        vhv_init_byte: u8,
    ) -> Result<(), Error<I::Error>> {
        write(i2c, SYSRANGE_START, 0x01 | vhv_init_byte).await?;
        poll(i2c, delay, RESULT_INTERRUPT_STATUS, |v| v & 0x07 != 0).await?;
        write(i2c, SYSTEM_INTERRUPT_CLEAR, 0x01).await?;
        write(i2c, SYSRANGE_START, 0x00).await
    }

    /// read timing budget
    async fn timing_budget_us<I: I2c>(&mut self, i2c: &mut I) -> Result<u32, Error<I::Error>> {
        let enables = sequence_step_enables(i2c).await?;
        let timeouts = sequence_step_timeouts(i2c, &enables).await?;
        let mut budget_us = START_OVERHEAD + END_OVERHEAD;
        if enables.tcc {
            budget_us += timeouts.msrc_dss_tcc_us + TCC_OVERHEAD;
        }
        if enables.dss {
            budget_us += 2 * (timeouts.msrc_dss_tcc_us + DSS_OVERHEAD);
        } else if enables.msrc {
            budget_us += timeouts.msrc_dss_tcc_us + MSRC_OVERHEAD;
        }
        if enables.pre_range {
            budget_us += timeouts.pre_range_us + PRE_RANGE_OVERHEAD;
        }
        if enables.final_range {
            budget_us += timeouts.final_range_us + FINAL_RANGE_OVERHEAD;
        }
        Ok(budget_us)
    }

    /// set timing budget
    async fn set_timing_budget_us<I: I2c>(
        &mut self,
        i2c: &mut I,
        budget_us: u32,
    ) -> Result<(), Error<I::Error>> {
        let enables = sequence_step_enables(i2c).await?;
        let timeouts = sequence_step_timeouts(i2c, &enables).await?;
        let mut used_budget_us = START_OVERHEAD + END_OVERHEAD;
        if enables.tcc {
            used_budget_us += timeouts.msrc_dss_tcc_us + TCC_OVERHEAD;
        }
        if enables.dss {
            used_budget_us += 2 * (timeouts.msrc_dss_tcc_us + DSS_OVERHEAD);
        } else if enables.msrc {
            used_budget_us += timeouts.msrc_dss_tcc_us + MSRC_OVERHEAD;
        }
        if enables.pre_range {
            used_budget_us += timeouts.pre_range_us + PRE_RANGE_OVERHEAD;
        }
        if enables.final_range {
            used_budget_us += FINAL_RANGE_OVERHEAD;
            // too small, skip
            if used_budget_us > budget_us {
                return Ok(());
            }
            let final_range_timeout_us = budget_us - used_budget_us;
            let mut final_range_timeout_mclks = timeout_us_to_mclks(
                final_range_timeout_us,
                timeouts.final_range_vcsel_period_pclks,
            );
            if enables.pre_range {
                final_range_timeout_mclks += u32::from(timeouts.pre_range_mclks);
            }
            write16(
                i2c,
                FINAL_RANGE_CONFIG_TIMEOUT_MACROP_HI,
                encode_timeout(final_range_timeout_mclks),
            )
            .await?;
        }
        Ok(())
    }
}

/// sequence step enables
async fn sequence_step_enables<I: I2c>(i2c: &mut I) -> Result<Enables, Error<I::Error>> {
    let config = read(i2c, SYSTEM_SEQUENCE_CONFIG).await?;
    Ok(Enables {
        tcc: (config >> 4) & 0x1 != 0,
        dss: (config >> 3) & 0x1 != 0,
        msrc: (config >> 2) & 0x1 != 0,
        pre_range: (config >> 6) & 0x1 != 0,
        final_range: (config >> 7) & 0x1 != 0,
    })
}

/// sequence step timeouts
async fn sequence_step_timeouts<I: I2c>(
    i2c: &mut I,
    enables: &Enables,
) -> Result<Timeouts, Error<I::Error>> {
    let pre_range_vcsel_period_pclks =
        decode_vcsel_period(read(i2c, PRE_RANGE_CONFIG_VCSEL_PERIOD).await?);
    let msrc_dss_tcc_mclks = u16::from(read(i2c, MSRC_CONFIG_TIMEOUT_MACROP).await?) + 1;
    let msrc_dss_tcc_us = timeout_mclks_to_us(msrc_dss_tcc_mclks, pre_range_vcsel_period_pclks);
    let pre_range_mclks = decode_timeout(read16(i2c, PRE_RANGE_CONFIG_TIMEOUT_MACROP_HI).await?);
    let pre_range_us = timeout_mclks_to_us(pre_range_mclks, pre_range_vcsel_period_pclks);
    let final_range_vcsel_period_pclks =
        decode_vcsel_period(read(i2c, FINAL_RANGE_CONFIG_VCSEL_PERIOD).await?);
    let mut final_range_mclks =
        decode_timeout(read16(i2c, FINAL_RANGE_CONFIG_TIMEOUT_MACROP_HI).await?);
    if enables.pre_range {
        final_range_mclks = final_range_mclks.wrapping_sub(pre_range_mclks);
    }
    let final_range_us = timeout_mclks_to_us(final_range_mclks, final_range_vcsel_period_pclks);
    Ok(Timeouts {
        final_range_vcsel_period_pclks,
        pre_range_mclks,
        msrc_dss_tcc_us,
        pre_range_us,
        final_range_us,
    })
}

fn decode_vcsel_period(reg: u8) -> u8 {
    ((u16::from(reg) + 1) << 1) as u8
}

/// macro period, ns
fn macro_period_ns(vcsel_period_pclks: u8) -> u32 {
    (2304 * u32::from(vcsel_period_pclks) * 1655 + 500) / 1000
}

fn timeout_mclks_to_us(mclks: u16, vcsel_period_pclks: u8) -> u32 {
    let macro_period_ns = macro_period_ns(vcsel_period_pclks);
    (u32::from(mclks).wrapping_mul(macro_period_ns) + 500) / 1000
}

fn timeout_us_to_mclks(us: u32, vcsel_period_pclks: u8) -> u32 {
    let macro_period_ns = macro_period_ns(vcsel_period_pclks);
    // avoid divide by zero
    (us.wrapping_mul(1000) + macro_period_ns / 2)
        .checked_div(macro_period_ns)
        .unwrap_or(0)
}

/// decode encoded timeout
fn decode_timeout(reg: u16) -> u16 {
    let lsb = u32::from(reg & 0x00FF);
    (lsb.checked_shl(u32::from(reg >> 8)).unwrap_or(0) as u16).wrapping_add(1)
}

fn encode_timeout(mclks: u32) -> u16 {
    if mclks == 0 {
        return 0;
    }
    let mut lsb = mclks - 1;
    let mut msb = 0u16;
    while lsb & 0xFFFF_FF00 > 0 {
        lsb >>= 1;
        msb += 1;
    }
    (msb << 8) | (lsb & 0xFF) as u16
}

async fn read<I: I2c>(i2c: &mut I, reg: u8) -> Result<u8, Error<I::Error>> {
    let mut value = [0u8; 1];
    read_into(i2c, reg, &mut value).await?;
    Ok(value[0])
}

async fn read16<I: I2c>(i2c: &mut I, reg: u8) -> Result<u16, Error<I::Error>> {
    let mut value = [0u8; 2];
    read_into(i2c, reg, &mut value).await?;
    Ok(u16::from_be_bytes(value))
}

/// pointer write, then read
async fn read_into<I: I2c>(i2c: &mut I, reg: u8, buf: &mut [u8]) -> Result<(), Error<I::Error>> {
    i2c.write(ADDRESS, &[reg]).await.map_err(Error::I2c)?;
    i2c.read(ADDRESS, buf).await.map_err(Error::I2c)
}

async fn write<I: I2c>(i2c: &mut I, reg: u8, value: u8) -> Result<(), Error<I::Error>> {
    i2c.write(ADDRESS, &[reg, value]).await.map_err(Error::I2c)
}

async fn write16<I: I2c>(i2c: &mut I, reg: u8, value: u16) -> Result<(), Error<I::Error>> {
    let [hi, lo] = value.to_be_bytes();
    i2c.write(ADDRESS, &[reg, hi, lo]).await.map_err(Error::I2c)
}

/// one transfer each
async fn write_all<I: I2c>(i2c: &mut I, writes: &[(u8, u8)]) -> Result<(), Error<I::Error>> {
    for &(reg, value) in writes {
        write(i2c, reg, value).await?;
    }
    Ok(())
}

/// poll until done
async fn poll<I: I2c>(
    i2c: &mut I,
    delay: &mut impl DelayNs,
    reg: u8,
    done: impl Fn(u8) -> bool,
) -> Result<(), Error<I::Error>> {
    for _ in 0..POLLS {
        if done(read(i2c, reg).await?) {
            return Ok(());
        }
        delay.delay_ms(1).await;
    }
    Err(Error::Timeout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeouts_round_trip_through_the_register_format() {
        // tuning timeout values
        assert_eq!(decode_timeout(0x0096), 151);
        assert_eq!(decode_timeout(0x01FE), 509);
        for mclks in [1, 151, 509, 1000, 4096, 30000] {
            let reg = encode_timeout(mclks);
            let back = u32::from(decode_timeout(reg));
            assert!(
                back <= mclks && mclks - back < back / 128 + 2,
                "{mclks} -> {reg:#06x} -> {back}"
            );
        }
    }

    #[test]
    fn macro_periods_match_the_api() {
        // tuning vcsel periods
        assert_eq!(decode_vcsel_period(0x06), 14);
        assert_eq!(decode_vcsel_period(0x04), 10);
        assert_eq!(macro_period_ns(14), 53_384);
        assert_eq!(macro_period_ns(10), 38_131);
    }
}
