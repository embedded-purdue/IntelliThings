//! air quality ratings

/// reading air state
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum State {
    Good = 0,
    Moderate = 1,
    Poor = 2,
    Bad = 3,
    /// informational neutral state
    Neutral = 4,
}

impl State {
    pub fn is_alert(self) -> bool {
        matches!(self, State::Moderate | State::Poor | State::Bad)
    }

    /// worst air state
    pub fn worst(self, other: State) -> State {
        match (self, other) {
            (State::Neutral, o) => o,
            (s, State::Neutral) => s,
            (a, b) => a.max(b),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Judgement {
    pub state: State,
    pub word: &'static str,
}

const fn j(state: State, word: &'static str) -> Judgement {
    Judgement { state, word }
}

pub fn temperature(c: f32) -> Judgement {
    match c {
        c if c < 16.0 => j(State::Poor, "Cold"),
        c if c < 19.0 => j(State::Moderate, "Cool"),
        c if c <= 25.0 => j(State::Good, "Comfortable"),
        c if c <= 28.0 => j(State::Moderate, "Warm"),
        c if c <= 32.0 => j(State::Poor, "Hot"),
        _ => j(State::Bad, "Very hot"),
    }
}

pub fn humidity(pct: f32) -> Judgement {
    match pct {
        p if p < 25.0 => j(State::Poor, "Very dry"),
        p if p < 30.0 => j(State::Moderate, "Dry"),
        p if p <= 60.0 => j(State::Good, "Comfortable"),
        p if p <= 70.0 => j(State::Moderate, "Humid"),
        _ => j(State::Poor, "Very humid"),
    }
}

/// co2 air state
pub fn co2(ppm: u16) -> Judgement {
    match ppm {
        0..800 => j(State::Good, "Fresh"),
        800..1200 => j(State::Moderate, "Getting stuffy"),
        1200..2000 => j(State::Poor, "Stuffy"),
        _ => j(State::Bad, "Ventilate now"),
    }
}

/// pm2.5 air state
pub fn pm2_5(ugm3: u16) -> Judgement {
    match ugm3 {
        0..12 => j(State::Good, "Clean"),
        12..36 => j(State::Moderate, "Particles up"),
        36..56 => j(State::Poor, "Poor"),
        _ => j(State::Bad, "Very poor"),
    }
}

/// voc air state
pub fn voc_index(index: u16) -> Judgement {
    match index {
        0..=150 => j(State::Good, "Low"),
        151..=250 => j(State::Moderate, "Elevated"),
        251..=400 => j(State::Poor, "High"),
        _ => j(State::Bad, "Very high"),
    }
}

/// illuminance air state
pub fn light(lux: f32) -> Judgement {
    match lux {
        l if l < 5.0 => j(State::Neutral, "Dark"),
        l if l < 150.0 => j(State::Neutral, "Dim"),
        l if l < 1000.0 => j(State::Neutral, "Bright"),
        _ => j(State::Neutral, "Very bright"),
    }
}

/// waiting sensor reading
pub fn waiting() -> Judgement {
    j(State::Neutral, "Waiting")
}

/// air state summary
pub fn summary(state: State) -> &'static str {
    match state {
        State::Good | State::Neutral => "Air fresh",
        State::Moderate => "Air fair",
        State::Poor => "Air poor",
        State::Bad => "Air bad",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn co2_matches_concept_bands() {
        assert_eq!(co2(642), j(State::Good, "Fresh"));
        assert_eq!(co2(1040), j(State::Moderate, "Getting stuffy"));
        assert_eq!(co2(1180), j(State::Moderate, "Getting stuffy"));
        assert_eq!(co2(1200).state, State::Poor);
        assert_eq!(co2(2400).state, State::Bad);
    }

    #[test]
    fn concept_values_read_as_shown() {
        assert_eq!(temperature((72.4 - 32.0) / 1.8).word, "Comfortable");
        assert_eq!(humidity(41.0).word, "Comfortable");
        assert_eq!(pm2_5(6).word, "Clean");
        assert_eq!(voc_index(118).word, "Low");
        assert_eq!(light(320.0).word, "Bright");
        assert_eq!(light(85.0).word, "Dim");
        assert_eq!(pm2_5(87).state, State::Bad);
    }

    #[test]
    fn neutral_never_wins() {
        assert_eq!(State::Neutral.worst(State::Good), State::Good);
        assert_eq!(State::Moderate.worst(State::Neutral), State::Moderate);
        assert_eq!(State::Moderate.worst(State::Bad), State::Bad);
    }
}
