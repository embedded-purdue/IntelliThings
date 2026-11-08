//! sensirion test vector

use sensors::voc_index::VocIndex;

/// same signal as c
struct Signal {
    seed: u32,
}

impl Signal {
    fn noise(&mut self) -> i32 {
        self.seed = self.seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        ((self.seed >> 16) % 201) as i32 - 100
    }

    fn at(&mut self, t: i32) -> u16 {
        let mut s = 30_000 + t / 60 + self.noise();
        if (2400..2700).contains(&t) {
            s -= 3000;
        }
        if (5000..5100).contains(&t) {
            s -= 6000;
        }
        if t == 3000 {
            s = 0;
        }
        if t == 3001 {
            s = 65_000;
        }
        s as u16
    }
}

#[test]
fn matches_sensirions_c_code_for_two_hours() {
    let expected: Vec<u16> = include_str!("fixtures/voc_reference.txt")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|line| line.parse().unwrap())
        .collect();
    assert_eq!(expected.len(), 7200);

    let mut voc = VocIndex::new();
    let mut signal = Signal { seed: 12_345 };
    let mismatches: Vec<(usize, u16, u16)> = expected
        .iter()
        .enumerate()
        .filter_map(|(t, &want)| {
            let got = voc.process(signal.at(t as i32));
            (got != want).then_some((t, got, want))
        })
        .collect();
    assert!(
        mismatches.is_empty(),
        "{} of 7200 seconds differ (second, ours, Sensirion's): {:?}",
        mismatches.len(),
        &mismatches[..mismatches.len().min(10)]
    );
}
