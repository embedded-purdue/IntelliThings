//! pololu driver test

mod sim;

use embassy_futures::block_on;
use sensors::vl53l0x::Vl53l0x;
use sim::{Clock, FakeVl53l0x, SimBus, SimDelay};

#[test]
fn every_transfer_matches_pololus_driver() {
    let clock = Clock::default();
    let mut bus = SimBus::new(&clock, vec![Box::new(FakeVl53l0x::new(500))]);
    bus.logging = true;
    let mut delay = SimDelay(clock.clone());
    let mut tof = Vl53l0x::default();
    block_on(tof.init(&mut bus, &mut delay)).expect("init");
    assert_eq!(
        block_on(tof.read_range_mm(&mut bus, &mut delay)).expect("range"),
        500
    );

    let pololu: Vec<&str> = include_str!("fixtures/vl53l0x_pololu.log")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .collect();
    for (i, (ours, theirs)) in bus.log.iter().zip(&pololu).enumerate() {
        assert_eq!(ours, theirs, "transfer {} of {}", i + 1, pololu.len());
    }
    assert_eq!(bus.log.len(), pololu.len(), "number of transfers");
}

#[test]
fn the_final_range_gets_the_time_msrc_and_tcc_gave_up() {
    // final range gets 0x0290
    let clock = Clock::default();
    let mut bus = SimBus::new(&clock, vec![Box::new(FakeVl53l0x::new(500))]);
    let mut delay = SimDelay(clock.clone());
    block_on(Vl53l0x::default().init(&mut bus, &mut delay)).expect("init");
    let chip = bus.device::<FakeVl53l0x>();
    assert_eq!([chip.reg(0x71), chip.reg(0x72)], [0x02, 0x90]);
    assert_eq!(chip.reg(0x01), 0xE8);
}
