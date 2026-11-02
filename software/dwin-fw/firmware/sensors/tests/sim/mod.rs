//! simulated sensor bus

#![allow(dead_code)]

use std::any::Any;
use std::cell::Cell;
use std::rc::Rc;

use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::i2c::{self, ErrorKind, ErrorType, I2c, NoAcknowledgeSource, Operation};

/// shared microsecond clock
#[derive(Clone, Default)]
pub struct Clock(Rc<Cell<u64>>);

impl Clock {
    pub fn now_us(&self) -> u64 {
        self.0.get()
    }

    pub fn advance_us(&self, us: u64) {
        self.0.set(self.0.get() + us);
    }

    /// like timer::at
    pub fn advance_to(&self, us: u64) {
        self.0.set(self.0.get().max(us));
    }
}

/// delays move the clock
pub struct SimDelay(pub Clock);

impl DelayNs for SimDelay {
    async fn delay_ns(&mut self, ns: u32) {
        self.0.advance_us(u64::from(ns).div_ceil(1000));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimError(pub ErrorKind);

impl i2c::Error for SimError {
    fn kind(&self) -> ErrorKind {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nack {
    Address,
    Data,
}

pub trait Device {
    fn address(&self) -> u8;
    /// bytes after address
    fn write(&mut self, now_us: u64, bytes: &[u8]) -> Result<(), Nack>;
    fn read(&mut self, now_us: u64, buf: &mut [u8]) -> Result<(), Nack>;
    fn as_any(&mut self) -> &mut dyn Any;
}

pub struct SimBus {
    pub clock: Clock,
    pub devices: Vec<Box<dyn Device>>,
    /// one line per transfer
    pub log: Vec<String>,
    pub logging: bool,
}

impl SimBus {
    pub fn new(clock: &Clock, devices: Vec<Box<dyn Device>>) -> Self {
        Self {
            clock: clock.clone(),
            devices,
            log: Vec::new(),
            logging: false,
        }
    }

    /// take off bus
    pub fn unplug(&mut self, address: u8) -> Box<dyn Device> {
        let i = self
            .devices
            .iter()
            .position(|d| d.address() == address)
            .expect("no such device");
        self.devices.remove(i)
    }

    pub fn plug(&mut self, device: Box<dyn Device>) {
        self.devices.push(device);
    }

    /// find fake by type
    pub fn device<T: 'static>(&mut self) -> &mut T {
        self.devices
            .iter_mut()
            .find_map(|d| d.as_any().downcast_mut::<T>())
            .expect("not on the bus")
    }
}

impl ErrorType for SimBus {
    type Error = SimError;
}

impl I2c for SimBus {
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), SimError> {
        let now = self.clock.now_us();
        let nack = |n: Nack| {
            SimError(ErrorKind::NoAcknowledge(match n {
                Nack::Address => NoAcknowledgeSource::Address,
                Nack::Data => NoAcknowledgeSource::Data,
            }))
        };
        let Some(device) = self.devices.iter_mut().find(|d| d.address() == address) else {
            return Err(nack(Nack::Address));
        };
        for op in operations {
            let (kind, bytes) = match op {
                Operation::Write(bytes) => {
                    device.write(now, bytes).map_err(nack)?;
                    ("W", &**bytes)
                }
                Operation::Read(buf) => {
                    device.read(now, buf).map_err(nack)?;
                    ("R", &**buf)
                }
            };
            if self.logging {
                let mut line = format!("{kind} {address:02X}");
                for b in bytes {
                    line += &format!(" {b:02X}");
                }
                self.log.push(line);
            }
        }
        Ok(())
    }
}

fn crc(hi: u8, lo: u8) -> u8 {
    let mut crc = 0xFF_u8;
    for byte in [hi, lo] {
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

/// words with crcs
fn words(words: &[u16]) -> Vec<u8> {
    words
        .iter()
        .flat_map(|w| {
            let [hi, lo] = w.to_be_bytes();
            [hi, lo, crc(hi, lo)]
        })
        .collect()
}

/// reply ready later
#[derive(Default)]
struct Pending(Option<(u64, Vec<u8>)>);

impl Pending {
    fn set(&mut self, ready_us: u64, reply: Vec<u8>) {
        self.0 = Some((ready_us, reply));
    }

    fn busy(&self, now: u64) -> bool {
        matches!(&self.0, Some((ready, _)) if now < *ready)
    }

    /// nack until ready
    fn take(&mut self, now: u64, buf: &mut [u8]) -> Result<(), Nack> {
        match self.0.take() {
            Some((ready, reply)) if now >= ready && buf.len() <= reply.len() => {
                buf.copy_from_slice(&reply[..buf.len()]);
                Ok(())
            }
            other => {
                self.0 = other;
                Err(Nack::Address)
            }
        }
    }
}

fn ticks(value: f32, offset: f32, span: f32) -> u16 {
    (((value + offset) / span).clamp(0.0, 1.0) * 65535.0).round() as u16
}

pub struct FakeSht41 {
    pub temperature_c: f32,
    pub humidity_pct: f32,
    pub measurements: u32,
    pending: Pending,
}

impl FakeSht41 {
    pub fn new(temperature_c: f32, humidity_pct: f32) -> Self {
        Self {
            temperature_c,
            humidity_pct,
            measurements: 0,
            pending: Pending::default(),
        }
    }
}

impl Device for FakeSht41 {
    fn address(&self) -> u8 {
        0x44
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn write(&mut self, now: u64, bytes: &[u8]) -> Result<(), Nack> {
        if self.pending.busy(now) {
            return Err(Nack::Address);
        }
        match bytes {
            [] => Ok(()),
            [0xFD] => {
                self.measurements += 1;
                let t = ticks(self.temperature_c, 45.0, 175.0);
                let rh = ticks(self.humidity_pct, 6.0, 125.0);
                self.pending.set(now + 8_300, words(&[t, rh]));
                Ok(())
            }
            [0x89] => {
                self.pending.set(now + 1_000, words(&[0x1A2B, 0x3C4D]));
                Ok(())
            }
            _ => Err(Nack::Data),
        }
    }

    fn read(&mut self, now: u64, buf: &mut [u8]) -> Result<(), Nack> {
        self.pending.take(now, buf)
    }
}

pub struct FakeSgp40 {
    pub sraw: u16,
    pub self_test_result: u16,
    /// last compensation ticks
    pub compensation: Option<[u16; 2]>,
    pub measurements: u32,
    pending: Pending,
}

impl FakeSgp40 {
    pub fn new(sraw: u16) -> Self {
        Self {
            sraw,
            self_test_result: 0xD400,
            compensation: None,
            measurements: 0,
            pending: Pending::default(),
        }
    }
}

impl Device for FakeSgp40 {
    fn address(&self) -> u8 {
        0x59
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn write(&mut self, now: u64, bytes: &[u8]) -> Result<(), Nack> {
        if self.pending.busy(now) {
            return Err(Nack::Address);
        }
        match *bytes {
            [] => Ok(()),
            [0x26, 0x0F, rh_hi, rh_lo, rh_crc, t_hi, t_lo, t_crc] => {
                if crc(rh_hi, rh_lo) != rh_crc || crc(t_hi, t_lo) != t_crc {
                    return Err(Nack::Data);
                }
                self.compensation = Some([
                    u16::from_be_bytes([rh_hi, rh_lo]),
                    u16::from_be_bytes([t_hi, t_lo]),
                ]);
                self.measurements += 1;
                self.pending.set(now + 30_000, words(&[self.sraw]));
                Ok(())
            }
            [0x28, 0x0E] => {
                self.pending
                    .set(now + 320_000, words(&[self.self_test_result]));
                Ok(())
            }
            [0x36, 0x82] => {
                self.pending
                    .set(now + 1_000, words(&[0x0000, 0x0123, 0x4567]));
                Ok(())
            }
            _ => Err(Nack::Data),
        }
    }

    fn read(&mut self, now: u64, buf: &mut [u8]) -> Result<(), Nack> {
        self.pending.take(now, buf)
    }
}

pub struct FakeScd41 {
    pub co2_ppm: u16,
    periodic: bool,
    started_us: u64,
    /// last read measurement
    consumed_us: Option<u64>,
    /// deaf after stop
    deaf_until_us: u64,
    pending: Pending,
}

const SCD41_PERIOD_US: u64 = 5_000_000;

impl FakeScd41 {
    /// idle after power-up
    pub fn new(co2_ppm: u16) -> Self {
        Self {
            co2_ppm,
            periodic: false,
            started_us: 0,
            consumed_us: None,
            deaf_until_us: 0,
            pending: Pending::default(),
        }
    }

    /// still measuring after reset
    pub fn measuring(co2_ppm: u16) -> Self {
        Self {
            periodic: true,
            ..Self::new(co2_ppm)
        }
    }

    /// lost power, now idle
    pub fn power_cycle(&mut self) {
        self.periodic = false;
        self.consumed_us = None;
    }

    fn latest_measurement(&self, now: u64) -> Option<u64> {
        let since = now.checked_sub(self.started_us)?;
        (self.periodic && since >= SCD41_PERIOD_US)
            .then(|| self.started_us + since / SCD41_PERIOD_US * SCD41_PERIOD_US)
    }
}

impl Device for FakeScd41 {
    fn address(&self) -> u8 {
        0x62
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn write(&mut self, now: u64, bytes: &[u8]) -> Result<(), Nack> {
        if now < self.deaf_until_us || self.pending.busy(now) {
            return Err(Nack::Address);
        }
        match bytes {
            [] => Ok(()),
            [0x3F, 0x86] => {
                self.periodic = false;
                self.deaf_until_us = now + 500_000;
                Ok(())
            }
            [0x21, 0xB1] if !self.periodic => {
                self.periodic = true;
                self.started_us = now;
                self.consumed_us = None;
                Ok(())
            }
            [0x36, 0x82] if !self.periodic => {
                self.pending
                    .set(now + 1_000, words(&[0xA1B2, 0xC3D4, 0xE5F6]));
                Ok(())
            }
            [0xE4, 0xB8] => {
                let ready = self
                    .latest_measurement(now)
                    .is_some_and(|m| Some(m) != self.consumed_us);
                self.pending
                    .set(now + 1_000, words(&[if ready { 0x8006 } else { 0x8000 }]));
                Ok(())
            }
            [0xEC, 0x05] => {
                let Some(m) = self.latest_measurement(now) else {
                    return Err(Nack::Data);
                };
                self.consumed_us = Some(m);
                // datasheet example t, rh
                self.pending
                    .set(now + 1_000, words(&[self.co2_ppm, 0x6667, 0x5EB9]));
                Ok(())
            }
            // refused while measuring
            _ => Err(Nack::Data),
        }
    }

    fn read(&mut self, now: u64, buf: &mut [u8]) -> Result<(), Nack> {
        self.pending.take(now, buf)
    }
}

pub struct FakeBh1750 {
    pub address: u8,
    pub lux: f32,
    powered: bool,
    measuring_since_us: Option<u64>,
}

impl FakeBh1750 {
    pub fn new(address: u8, lux: f32) -> Self {
        Self {
            address,
            lux,
            powered: false,
            measuring_since_us: None,
        }
    }
}

impl Device for FakeBh1750 {
    fn address(&self) -> u8 {
        self.address
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn write(&mut self, now: u64, bytes: &[u8]) -> Result<(), Nack> {
        match bytes {
            [] => {}
            [0x01] => self.powered = true,
            [0x10] if self.powered => self.measuring_since_us = Some(now),
            _ => return Err(Nack::Data),
        }
        Ok(())
    }

    fn read(&mut self, now: u64, buf: &mut [u8]) -> Result<(), Nack> {
        // zero until 180 ms
        let ready = self.measuring_since_us.is_some_and(|t| now >= t + 180_000);
        let count = if ready {
            (self.lux * 1.2).round() as u16
        } else {
            0
        };
        buf.copy_from_slice(&count.to_be_bytes()[..buf.len()]);
        Ok(())
    }
}

/// same model as c++
pub struct FakeVl53l0x {
    regs: Vec<[u8; 256]>,
    page: u8,
    ptr: u8,
    spad_busy: bool,
    int_busy_reads: u8,
}

impl FakeVl53l0x {
    pub fn new(range_mm: u16) -> Self {
        let mut regs = vec![[0u8; 256]; 256];
        regs[0][0xC0] = 0xEE;
        regs[0][0x84] = 0x11;
        regs[0][0xB0..0xB6].copy_from_slice(&[0xA5, 0x5A, 0xFF, 0x0F, 0xF0, 0x3C]);
        regs[0][0x14] = 0x58;
        regs[0][0x1E..0x20].copy_from_slice(&range_mm.to_be_bytes());
        regs[1][0x91] = 0x3C;
        regs[7][0x92] = 0x89;
        Self {
            regs,
            page: 0,
            ptr: 0,
            spad_busy: false,
            int_busy_reads: 0,
        }
    }

    pub fn set_range_mm(&mut self, mm: u16) {
        self.regs[0][0x1E..0x20].copy_from_slice(&mm.to_be_bytes());
    }

    /// page 0 register
    pub fn reg(&self, addr: u8) -> u8 {
        self.regs[0][addr as usize]
    }

    fn store(&mut self, addr: u8, value: u8) {
        if addr == 0xFF {
            self.page = value;
            return;
        }
        self.regs[self.page as usize][addr as usize] = value;
        if self.page == 7 && addr == 0x83 && value == 0x00 {
            self.spad_busy = true;
        }
        if self.page == 0 && addr == 0x00 && value & 0x01 != 0 {
            self.int_busy_reads = 1;
            self.regs[0][0x13] = 0x00;
        }
        if self.page == 0 && addr == 0x0B && value & 0x01 != 0 {
            self.regs[0][0x13] = 0x00;
        }
    }

    fn load(&mut self, addr: u8) -> u8 {
        if addr == 0xFF {
            return self.page;
        }
        if self.page == 7 && addr == 0x83 && self.spad_busy {
            self.spad_busy = false;
            self.regs[7][0x83] = 0x01;
            return 0x00;
        }
        if self.page == 0 && addr == 0x13 && self.int_busy_reads > 0 {
            self.int_busy_reads -= 1;
            if self.int_busy_reads == 0 {
                self.regs[0][0x13] = 0x07;
            }
            return 0x00;
        }
        if self.page == 0 && addr == 0x00 {
            let v = self.regs[0][0x00];
            self.regs[0][0x00] &= !0x01;
            return v;
        }
        self.regs[self.page as usize][addr as usize]
    }
}

impl Device for FakeVl53l0x {
    fn address(&self) -> u8 {
        0x29
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn write(&mut self, _now: u64, bytes: &[u8]) -> Result<(), Nack> {
        if let [reg, values @ ..] = bytes {
            self.ptr = *reg;
            for (i, &v) in values.iter().enumerate() {
                self.store(reg.wrapping_add(i as u8), v);
            }
        }
        Ok(())
    }

    fn read(&mut self, _now: u64, buf: &mut [u8]) -> Result<(), Nack> {
        for b in buf {
            *b = self.load(self.ptr);
            self.ptr = self.ptr.wrapping_add(1);
        }
        Ok(())
    }
}

/// all five sensors
pub fn board(clock: &Clock) -> SimBus {
    SimBus::new(
        clock,
        vec![
            Box::new(FakeSht41::new(22.4, 41.2)),
            Box::new(FakeSgp40::new(30_000)),
            Box::new(FakeScd41::new(684)),
            Box::new(FakeBh1750::new(0x23, 312.5)),
            Box::new(FakeVl53l0x::new(845)),
        ],
    )
}
