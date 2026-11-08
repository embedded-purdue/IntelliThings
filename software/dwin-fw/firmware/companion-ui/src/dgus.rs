//! dgus serial protocol

use embedded_io_async::Write;

pub const HEADER: [u8; 2] = [0x5A, 0xA5];
pub const CMD_WRITE: u8 = 0x82;
pub const CMD_READ: u8 = 0x83;

/// max write bytes
pub const MAX_WRITE_BYTES: usize = 240;

/// system registers
pub mod sys {
    /// dgus core version
    pub const VERSION: u16 = 0x000F;
    /// current page id
    pub const PIC_NOW: u16 = 0x0014;
    /// backlight level config
    pub const LED_CONFIG: u16 = 0x0082;
    /// switch page id
    pub const PIC_SET: u16 = 0x0084;
    /// panel reboot register
    pub const SYS_RESET: u16 = 0x0004;
}

/// modbus crc16
pub fn crc16(bytes: &[u8]) -> u16 {
    let mut crc = 0xFFFFu16;
    for &b in bytes {
        crc ^= b as u16;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xA001
            } else {
                crc >> 1
            };
        }
    }
    crc
}

/// encode write frame
pub fn encode_write(out: &mut [u8], vp: u16, data: &[u8], crc: bool) -> usize {
    assert!(data.len() <= MAX_WRITE_BYTES && data.len().is_multiple_of(2));
    let mut n = 6 + data.len();
    out[..2].copy_from_slice(&HEADER);
    out[2] = (3 + data.len() + if crc { 2 } else { 0 }) as u8;
    out[3] = CMD_WRITE;
    out[4..6].copy_from_slice(&vp.to_be_bytes());
    out[6..n].copy_from_slice(data);
    if crc {
        let c = crc16(&out[3..n]);
        out[n..n + 2].copy_from_slice(&c.to_le_bytes());
        n += 2;
    }
    n
}

/// encode read request
pub fn encode_read(vp: u16, words: u8, crc: bool) -> ([u8; 9], usize) {
    let a = vp.to_be_bytes();
    let mut f = [HEADER[0], HEADER[1], 4, CMD_READ, a[0], a[1], words, 0, 0];
    if !crc {
        return (f, 7);
    }
    f[2] = 6;
    let c = crc16(&f[3..7]).to_le_bytes();
    f[7] = c[0];
    f[8] = c[1];
    (f, 9)
}

/// panel serial link
pub struct Link<W> {
    pub io: W,
    /// append crc flag
    pub crc: bool,
    /// sent frame count
    pub frames: u32,
}

impl<W: Write> Link<W> {
    pub fn new(io: W) -> Self {
        Self {
            io,
            crc: false,
            frames: 0,
        }
    }

    /// write vp words
    pub async fn write_words(&mut self, vp: u16, words: &[u16]) -> Result<(), W::Error> {
        let mut frame = [0u8; 8 + MAX_WRITE_BYTES];
        let mut data = [0u8; MAX_WRITE_BYTES];
        let per = MAX_WRITE_BYTES / 2;
        for (k, chunk) in words.chunks(per).enumerate() {
            for (i, w) in chunk.iter().enumerate() {
                data[2 * i..2 * i + 2].copy_from_slice(&w.to_be_bytes());
            }
            let n = encode_write(
                &mut frame,
                vp + (k * per) as u16,
                &data[..2 * chunk.len()],
                self.crc,
            );
            self.io.write_all(&frame[..n]).await?;
            self.frames = self.frames.wrapping_add(1);
        }
        Ok(())
    }

    pub async fn request_read(&mut self, vp: u16, words: u8) -> Result<(), W::Error> {
        let (f, n) = encode_read(vp, words, self.crc);
        self.io.write_all(&f[..n]).await
    }

    pub async fn set_page(&mut self, page: u16) -> Result<(), W::Error> {
        self.write_words(sys::PIC_SET, &[0x5A01, page]).await
    }

    /// set backlight level
    pub async fn set_backlight(&mut self, level: u8) -> Result<(), W::Error> {
        let l = level.min(100) as u16;
        self.write_words(sys::LED_CONFIG, &[(l << 8) | l, 0]).await
    }

    pub async fn reset(&mut self) -> Result<(), W::Error> {
        self.write_words(sys::SYS_RESET, &[0x55AA, 0x5AA5]).await
    }
}

/// panel reply frame
#[derive(Debug, PartialEq, Eq)]
pub enum Reply<'a> {
    /// write ack frame
    Ack,
    /// read data words
    Data {
        vp: u16,
        words: &'a [u8],
    },
    Other(&'a [u8]),
}

/// parse reply frame
pub fn parse_reply(buf: &[u8]) -> Option<(Reply<'_>, usize)> {
    let start = buf.windows(2).position(|w| w == HEADER)?;
    let len = *buf.get(start + 2)? as usize;
    let end = start + 3 + len;
    let body = buf.get(start + 3..end)?;
    let reply = match body {
        [CMD_WRITE, 0x4F, 0x4B, ..] => Reply::Ack,
        [CMD_READ, hi, lo, n, rest @ ..] if rest.len() >= 2 * *n as usize => Reply::Data {
            vp: u16::from_be_bytes([*hi, *lo]),
            words: &rest[..2 * *n as usize],
        },
        other => Reply::Other(other),
    };
    Some((reply, end))
}

/// vp shadow image
pub struct VpImage<const N: usize> {
    pub base: u16,
    want: [u16; N],
    have: [u16; N],
    known: [bool; N],
}

impl<const N: usize> VpImage<N> {
    pub const fn new(base: u16) -> Self {
        Self {
            base,
            want: [0; N],
            have: [0; N],
            known: [false; N],
        }
    }

    fn index(&self, vp: u16) -> usize {
        let i = vp.wrapping_sub(self.base) as usize;
        assert!(i < N, "VP {vp:#06x} outside the image");
        i
    }

    pub fn set(&mut self, vp: u16, value: u16) {
        let i = self.index(vp);
        self.want[i] = value;
    }

    pub fn set_slice(&mut self, vp: u16, values: &[u16]) {
        let i = self.index(vp);
        self.want[i..i + values.len()].copy_from_slice(values);
    }

    pub fn get(&self, vp: u16) -> u16 {
        self.want[self.index(vp)]
    }

    /// target word buffer
    pub fn words(&self) -> &[u16; N] {
        &self.want
    }

    /// invalidate panel shadow
    pub fn invalidate(&mut self) {
        self.known = [false; N];
    }

    fn dirty(&self, i: usize) -> bool {
        !self.known[i] || self.want[i] != self.have[i]
    }

    /// changed word runs
    pub fn dirty_runs(&self, mut f: impl FnMut(usize, usize)) {
        const MERGE_GAP: usize = 3;
        let mut i = 0;
        while i < N {
            if !self.dirty(i) {
                i += 1;
                continue;
            }
            let start = i;
            let mut end = i + 1; // dirty end index
            let mut j = end;
            while j < N && j - end <= MERGE_GAP {
                if self.dirty(j) {
                    end = j + 1;
                }
                j += 1;
            }
            f(start, end - start);
            i = end;
        }
    }

    /// flush changed words
    pub async fn flush<W: Write>(&mut self, link: &mut Link<W>) -> Result<usize, W::Error> {
        let mut runs: heapless::Vec<(usize, usize), 64> = heapless::Vec::new();
        let mut overflow = false;
        self.dirty_runs(|s, l| {
            if runs.push((s, l)).is_err() {
                overflow = true;
            }
        });
        let mut sent = 0;
        for &(s, l) in &runs {
            link.write_words(self.base + s as u16, &self.want[s..s + l])
                .await?;
            self.have[s..s + l].copy_from_slice(&self.want[s..s + l]);
            self.known[s..s + l].iter_mut().for_each(|k| *k = true);
            sent += 6 * l.div_ceil(MAX_WRITE_BYTES / 2) + 2 * l;
        }
        if overflow {
            // next flush remainder
        }
        Ok(sent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    /// test byte sink
    pub struct Sink(pub Vec<u8>);

    impl embedded_io_async::ErrorType for Sink {
        type Error = core::convert::Infallible;
    }

    impl Write for Sink {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.0.extend_from_slice(buf);
            Ok(buf.len())
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    pub fn block_on<F: core::future::Future>(f: F) -> F::Output {
        use core::pin::pin;
        use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
        fn noop(_: *const ()) {}
        fn clone(p: *const ()) -> RawWaker {
            RawWaker::new(p, &VTABLE)
        }
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
        let waker = unsafe { Waker::from_raw(RawWaker::new(core::ptr::null(), &VTABLE)) };
        let mut cx = Context::from_waker(&waker);
        let mut f = pin!(f);
        loop {
            if let Poll::Ready(v) = f.as_mut().poll(&mut cx) {
                return v;
            }
        }
    }

    #[test]
    fn page_switch_frame() {
        let mut l = Link::new(Sink(Vec::new()));
        block_on(l.set_page(3)).unwrap();
        assert_eq!(
            l.io.0,
            [0x5A, 0xA5, 0x07, 0x82, 0x00, 0x84, 0x5A, 0x01, 0x00, 0x03]
        );
    }

    #[test]
    fn crc_matches_the_guide_examples() {
        // guide chapter 4.1
        let (f, n) = encode_read(0x000F, 1, true);
        assert_eq!(
            &f[..n],
            &[0x5A, 0xA5, 0x06, 0x83, 0x00, 0x0F, 0x01, 0xED, 0x90]
        );
        let mut out = [0u8; 16];
        let n = encode_write(&mut out, 0x1000, &[0x31, 0x32], true);
        assert_eq!(
            &out[..n],
            &[0x5A, 0xA5, 0x07, 0x82, 0x10, 0x00, 0x31, 0x32, 0xCC, 0x9B]
        );
    }

    #[test]
    fn long_writes_are_split() {
        let mut l = Link::new(Sink(Vec::new()));
        let words: Vec<u16> = (0..200).collect();
        block_on(l.write_words(0x5000, &words)).unwrap();
        let s = &l.io;
        // chunked word writes
        assert_eq!(&s.0[..6], &[0x5A, 0xA5, 243, 0x82, 0x50, 0x00]);
        let second = 6 + 240;
        assert_eq!(
            &s.0[second..second + 6],
            &[0x5A, 0xA5, 163, 0x82, 0x50, 0x78]
        );
        assert_eq!(s.0.len(), second + 6 + 160);
    }

    #[test]
    fn replies() {
        let buf = [
            0x00, 0x5A, 0xA5, 0x03, 0x82, 0x4F, 0x4B, 0x5A, 0xA5, 0x06, 0x83, 0x00, 0x14, 0x01,
            0x00, 0x02,
        ];
        let (r, used) = parse_reply(&buf).unwrap();
        assert_eq!((r, used), (Reply::Ack, 7));
        let (r, used) = parse_reply(&buf[7..]).unwrap();
        assert_eq!(
            r,
            Reply::Data {
                vp: 0x0014,
                words: &[0x00, 0x02]
            }
        );
        assert_eq!(used, 9);
        assert!(parse_reply(&buf[7..12]).is_none());
    }

    #[test]
    fn image_sends_only_changes() {
        let mut img: VpImage<64> = VpImage::new(0x5000);
        let mut l = Link::new(Sink(Vec::new()));
        block_on(img.flush(&mut l)).unwrap();
        assert_eq!(l.io.0.len(), 6 + 128); // initial full flush
        l.io.0.clear();
        block_on(img.flush(&mut l)).unwrap();
        assert!(l.io.0.is_empty());
        img.set(0x5002, 7);
        img.set(0x5004, 9); // merged gap write
        img.set(0x5030, 1);
        block_on(img.flush(&mut l)).unwrap();
        assert_eq!(l.io.0.len(), (6 + 6) + (6 + 2));
        assert_eq!(&l.io.0[4..6], &[0x50, 0x02]);
        img.invalidate();
        l.io.0.clear();
        block_on(img.flush(&mut l)).unwrap();
        assert_eq!(l.io.0.len(), 6 + 128);
    }
}
