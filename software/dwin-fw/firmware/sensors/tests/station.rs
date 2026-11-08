//! simulated board readout

mod sim;

use embassy_futures::block_on;
use sensors::Station;
use sim::{Clock, FakeBh1750, FakeScd41, FakeSgp40, FakeSht41, FakeVl53l0x, SimBus, SimDelay};

const SECOND_US: u64 = 1_000_000;

/// ticks like the firmware
struct Run {
    clock: Clock,
    bus: SimBus,
    delay: SimDelay,
    station: Station,
    out: String,
    started_us: u64,
    ticks: u64,
}

impl Run {
    fn start(bus: SimBus, clock: Clock) -> Self {
        let mut run = Run {
            delay: SimDelay(clock.clone()),
            clock,
            bus,
            station: Station::new(),
            out: String::new(),
            started_us: 0,
            ticks: 0,
        };
        block_on(
            run.station
                .start(&mut run.bus, &mut run.delay, &mut run.out),
        )
        .unwrap();
        run.started_us = run.clock.now_us();
        run
    }

    fn board() -> Self {
        let clock = Clock::default();
        Run::start(sim::board(&clock), clock)
    }

    /// run n seconds
    fn seconds(&mut self, n: u64) -> String {
        let from = self.out.len();
        for _ in 0..n {
            self.ticks += 1;
            self.clock
                .advance_to(self.started_us + self.ticks * SECOND_US);
            block_on(
                self.station
                    .tick(&mut self.bus, &mut self.delay, &mut self.out),
            )
            .unwrap();
        }
        self.out[from..].to_string()
    }

    /// last row's cells
    fn last_row(&self) -> Vec<String> {
        let row = self.out.lines().last().unwrap();
        row.split_whitespace().map(str::to_string).collect()
    }
}

#[test]
fn all_five_come_up_and_fill_the_table() {
    let mut run = Run::board();
    let boot = run.out.clone();
    assert!(
        boot.contains(
            "I2C scan: 0x23 BH1750, 0x29 VL53L0X, 0x44 SHT41, 0x59 SGP40, 0x62 SCD41 (5 devices)"
        ),
        "{boot}"
    );
    assert!(
        boot.contains("  SHT41    0x44  ok, serial 1A2B3C4D\n"),
        "{boot}"
    );
    assert!(
        boot.contains("  SGP40    0x59  ok, self-test passed, serial 000001234567\n"),
        "{boot}"
    );
    assert!(
        boot.contains("  SCD41    0x62  ok, serial A1B2C3D4E5F6\n"),
        "{boot}"
    );
    assert!(boot.contains("  BH1750   0x23  ok\n"), "{boot}");
    assert!(boot.contains("  VL53L0X  0x29  ok\n"), "{boot}");

    // no co2, voc yet
    let first = run.seconds(1);
    assert!(
        first.contains("sensors: SHT41 ok, SGP40 ok, SCD41 ok, BH1750 ok, VL53L0X ok"),
        "{first}"
    );
    assert!(
        first.contains(
            " uptime  temperature_c  humidity_pct  co2_ppm  voc_index       lux  distance_mm\n"
        ),
        "{first}"
    );
    assert_eq!(
        run.last_row(),
        ["0:01", "22.4", "41.2", "wait", "wait", "312.5", "845"]
    );

    // co2 after 5 s
    run.seconds(5);
    assert_eq!(run.last_row()[3], "684");
    assert_eq!(run.station.snapshot().co2_ppm, Some(684));

    // voc after 45 s
    run.seconds(54);
    let row = run.last_row();
    assert_eq!(row[0], "1:00");
    let voc: u16 = row[4].parse().expect("a VOC index by 1:00");
    assert!((1..=500).contains(&voc));
    let snapshot = run.station.snapshot();
    assert_eq!(
        snapshot.temperature_c.map(|t| (t * 10.0).round() / 10.0),
        Some(22.4)
    );
    assert_eq!(
        snapshot.humidity_pct.map(|h| (h * 10.0).round() / 10.0),
        Some(41.2)
    );
    assert_eq!(snapshot.lux, Some(312.5));
    assert_eq!(snapshot.distance_mm, Some(845));
    assert_eq!(snapshot.voc_index, Some(voc));
}

#[test]
fn the_sgp40_is_compensated_with_the_sht41_reading() {
    let mut run = Run::board();
    run.seconds(3);
    // sht41 values as ticks
    let expected = [
        (0.412_f32 * 65535.0).round() as u16,
        ((22.4_f32 + 45.0) / 175.0 * 65535.0).round() as u16,
    ];
    let sgp = run.bus.device::<FakeSgp40>();
    let got = sgp.compensation.unwrap();
    assert!(
        got[0].abs_diff(expected[0]) <= 2 && got[1].abs_diff(expected[1]) <= 2,
        "{got:?} vs {expected:?}"
    );
    assert_eq!(sgp.measurements, 3, "one sample a second");
}

#[test]
fn an_hour_in_steady_air_reads_about_100() {
    let mut run = Run::board();
    let printed = run.seconds(3600);
    assert_eq!(run.last_row()[0], "1:00:00");
    let voc: u16 = run.last_row()[4].parse().unwrap();
    assert!((90..=110).contains(&voc), "VOC index {voc}");
    // header every 30 rows
    assert_eq!(printed.matches("\nsensors:").count(), 120);
    assert!(!printed.contains("lost"), "{}", &printed[..2000]);
}

#[test]
fn a_missing_sensor_shows_dashes_and_is_found_when_plugged_in() {
    let clock = Clock::default();
    let mut bus = sim::board(&clock);
    let bh1750 = bus.unplug(0x23);
    let mut run = Run::start(bus, clock);
    assert!(run.out.contains("  BH1750   0x23  not found (no answer at its address): check its wires. Retrying every 10 s.\n"), "{}", run.out);
    run.seconds(3);
    assert_eq!(run.last_row()[5], "--");
    assert!(
        run.out
            .contains("sensors: SHT41 ok, SGP40 ok, SCD41 ok, BH1750 not answering, VL53L0X ok")
    );
    assert_eq!(run.station.snapshot().lux, None);

    // plugged in at 0:12
    run.seconds(9);
    run.bus.plug(bh1750);
    let printed = run.seconds(9);
    assert!(printed.contains("[0:20] BH1750 found\n"), "{printed}");
    assert_eq!(run.last_row()[0], "0:21");
    assert_eq!(run.last_row()[5], "312.5");
}

#[test]
fn a_sensor_unplugged_mid_run_is_lost_then_found() {
    let mut run = Run::board();
    run.seconds(10);
    let sht41 = run.bus.unplug(0x44);
    // lost after three misses
    let printed = run.seconds(3);
    assert!(
        printed.contains("[0:13] SHT41 lost (no answer at its address). Retrying every 10 s.\n"),
        "{printed}"
    );
    assert_eq!(run.last_row()[1..3], ["--", "--"]);
    assert_eq!(run.station.snapshot().temperature_c, None);

    run.bus.plug(sht41);
    let printed = run.seconds(11);
    assert!(
        printed.contains("[0:23] SHT41 found, serial 1A2B3C4D\n"),
        "{printed}"
    );
    assert_eq!(run.last_row()[1], "22.4");
}

#[test]
fn an_scd41_still_measuring_from_before_a_reset_is_stopped_and_restarted() {
    let clock = Clock::default();
    let mut bus = sim::board(&clock);
    *bus.device::<FakeScd41>() = FakeScd41::measuring(950);
    let mut run = Run::start(bus, clock);
    assert!(
        run.out
            .contains("  SCD41    0x62  ok, serial A1B2C3D4E5F6\n"),
        "{}",
        run.out
    );
    run.seconds(6);
    assert_eq!(run.last_row()[3], "950");
}

#[test]
fn an_scd41_that_lost_power_is_noticed_and_restarted() {
    let mut run = Run::board();
    run.seconds(20);
    assert_eq!(run.last_row()[3], "684");
    // idle, so goes stale
    run.bus.device::<FakeScd41>().power_cycle();
    let printed = run.seconds(40);
    assert!(
        printed.contains("SCD41 lost (timed out). Retrying every 10 s.\n"),
        "{printed}"
    );
    assert!(
        printed.contains("SCD41 found, serial A1B2C3D4E5F6\n"),
        "{printed}"
    );
    assert_eq!(run.last_row()[3], "684");
}

#[test]
fn nothing_in_range_reads_far() {
    let mut run = Run::board();
    run.bus.device::<FakeVl53l0x>().set_range_mm(8190);
    run.seconds(1);
    assert_eq!(run.last_row()[6], "far");
    assert_eq!(run.station.snapshot().distance_mm, None);
    run.bus.device::<FakeVl53l0x>().set_range_mm(120);
    run.seconds(1);
    assert_eq!(run.last_row()[6], "120");
}

#[test]
fn a_bh1750_with_addr_high_is_found_at_0x5c() {
    let clock = Clock::default();
    let mut bus = sim::board(&clock);
    bus.unplug(0x23);
    bus.plug(Box::new(FakeBh1750::new(0x5C, 40.0)));
    let mut run = Run::start(bus, clock);
    assert!(run.out.contains("0x5C BH1750"), "{}", run.out);
    assert!(run.out.contains("  BH1750   0x5C  ok\n"), "{}", run.out);
    run.seconds(1);
    assert_eq!(run.last_row()[5], "40.0");
}

#[test]
fn a_failed_sgp40_self_test_counts_as_not_found() {
    let clock = Clock::default();
    let mut bus = sim::board(&clock);
    bus.device::<FakeSgp40>().self_test_result = 0x4B00;
    let run = Run::start(bus, clock);
    assert!(
        run.out
            .contains("  SGP40    0x59  not found (unexpected reply)"),
        "{}",
        run.out
    );
}

#[test]
fn an_empty_bus_says_what_to_check() {
    let clock = Clock::default();
    let mut run = Run::start(SimBus::new(&clock, vec![]), clock);
    assert!(
        run.out.contains(
            "I2C scan: nothing answered. Check SDA, SCL, and the sensors' power and ground.\n"
        ),
        "{}",
        run.out
    );
    run.seconds(1);
    assert_eq!(run.last_row(), ["0:01", "--", "--", "--", "--", "--", "--"]);
}

#[test]
fn the_sht41_is_read_after_its_measurement_time() {
    // nacks before 8.3 ms
    let mut run = Run::board();
    run.seconds(5);
    assert_eq!(run.bus.device::<FakeSht41>().measurements, 5);
    assert_eq!(run.last_row()[1], "22.4");
}
