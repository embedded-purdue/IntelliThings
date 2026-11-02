//! sensirion voc index
// bsd-3: see licenses.md

use libm::{expf, sqrtf};

const SAMPLING_INTERVAL: f32 = 1.0;
const INITIAL_BLACKOUT: f32 = 45.0;
const INDEX_GAIN: f32 = 230.0;
const SRAW_STD_INITIAL: f32 = 50.0;
const SRAW_STD_BONUS: f32 = 220.0;
const TAU_MEAN_HOURS: f32 = 12.0;
const TAU_VARIANCE_HOURS: f32 = 12.0;
const TAU_INITIAL_MEAN: f32 = 20.0;
const INIT_DURATION_MEAN: f32 = 3600.0 * 0.75;
const INIT_TRANSITION_MEAN: f32 = 0.01;
const TAU_INITIAL_VARIANCE: f32 = 2500.0;
const INIT_DURATION_VARIANCE: f32 = 3600.0 * 1.45;
const INIT_TRANSITION_VARIANCE: f32 = 0.01;
const GATING_THRESHOLD: f32 = 340.0;
const GATING_THRESHOLD_INITIAL: f32 = 510.0;
const GATING_THRESHOLD_TRANSITION: f32 = 0.09;
const GATING_MAX_DURATION_MINUTES: f32 = 60.0 * 3.0;
const GATING_MAX_RATIO: f32 = 0.3;
const SIGMOID_L: f32 = 500.0;
const SIGMOID_K: f32 = -0.0065;
const SIGMOID_X0: f32 = 213.0;
const INDEX_OFFSET: f32 = 100.0;
const LP_TAU_FAST: f32 = 20.0;
const LP_TAU_SLOW: f32 = 500.0;
const LP_ALPHA: f32 = -0.2;
const SRAW_MINIMUM: i32 = 20000;
const GAMMA_SCALING: f32 = 64.0;
const ADDITIONAL_GAMMA_MEAN_SCALING: f32 = 8.0;
const FIX16_MAX: f32 = 32767.0;

/// algorithm state
#[derive(Debug, Clone)]
pub struct VocIndex {
    uptime: f32,
    sraw: f32,
    gas_index: f32,
    // mean_variance_estimator
    mve_initialized: bool,
    mve_mean: f32,
    mve_sraw_offset: f32,
    mve_std: f32,
    mve_gamma_mean: f32,
    mve_gamma_variance: f32,
    mve_gamma_initial_mean: f32,
    mve_gamma_initial_variance: f32,
    mve_gamma_mean_now: f32,
    mve_gamma_variance_now: f32,
    mve_uptime_gamma: f32,
    mve_uptime_gating: f32,
    mve_gating_duration_minutes: f32,
    // mox_model
    mox_sraw_std: f32,
    mox_sraw_mean: f32,
    // adaptive_lowpass
    lp_a1: f32,
    lp_a2: f32,
    lp_initialized: bool,
    lp_x1: f32,
    lp_x2: f32,
    lp_x3: f32,
}

impl Default for VocIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl VocIndex {
    /// gas index init, voc
    pub fn new() -> Self {
        let interval_hours = SAMPLING_INTERVAL / 3600.0;
        let mut voc = Self {
            uptime: 0.0,
            sraw: 0.0,
            gas_index: 0.0,
            // mean_variance_estimator__set_parameters
            mve_initialized: false,
            mve_mean: 0.0,
            mve_sraw_offset: 0.0,
            mve_std: SRAW_STD_INITIAL,
            mve_gamma_mean: ((ADDITIONAL_GAMMA_MEAN_SCALING * GAMMA_SCALING) * interval_hours)
                / (TAU_MEAN_HOURS + interval_hours),
            mve_gamma_variance: (GAMMA_SCALING * interval_hours)
                / (TAU_VARIANCE_HOURS + interval_hours),
            mve_gamma_initial_mean: ((ADDITIONAL_GAMMA_MEAN_SCALING * GAMMA_SCALING)
                * SAMPLING_INTERVAL)
                / (TAU_INITIAL_MEAN + SAMPLING_INTERVAL),
            mve_gamma_initial_variance: (GAMMA_SCALING * SAMPLING_INTERVAL)
                / (TAU_INITIAL_VARIANCE + SAMPLING_INTERVAL),
            mve_gamma_mean_now: 0.0,
            mve_gamma_variance_now: 0.0,
            mve_uptime_gamma: 0.0,
            mve_uptime_gating: 0.0,
            mve_gating_duration_minutes: 0.0,
            mox_sraw_std: 0.0,
            mox_sraw_mean: 0.0,
            // adaptive_lowpass__set_parameters
            lp_a1: SAMPLING_INTERVAL / (LP_TAU_FAST + SAMPLING_INTERVAL),
            lp_a2: SAMPLING_INTERVAL / (LP_TAU_SLOW + SAMPLING_INTERVAL),
            lp_initialized: false,
            lp_x1: 0.0,
            lp_x2: 0.0,
            lp_x3: 0.0,
        };
        voc.mox_set_parameters();
        voc
    }

    /// raw sample to index
    pub fn process(&mut self, sraw: u16) -> u16 {
        let sraw = i32::from(sraw);
        if self.uptime <= INITIAL_BLACKOUT {
            self.uptime += SAMPLING_INTERVAL;
        } else {
            if sraw > 0 && sraw < 65000 {
                let sraw = sraw.clamp(SRAW_MINIMUM + 1, SRAW_MINIMUM + 32767);
                self.sraw = (sraw - SRAW_MINIMUM) as f32;
            }
            self.gas_index = self.mox_process(self.sraw);
            self.gas_index = sigmoid_scaled(self.gas_index);
            self.gas_index = self.lowpass(self.gas_index);
            if self.gas_index < 0.5 {
                self.gas_index = 0.5;
            }
            if self.sraw > 0.0 {
                self.mve_process(self.sraw);
                self.mox_set_parameters();
            }
        }
        (self.gas_index + 0.5) as u16
    }

    /// mox_model__set_parameters
    fn mox_set_parameters(&mut self) {
        self.mox_sraw_std = self.mve_std;
        self.mox_sraw_mean = self.mve_mean + self.mve_sraw_offset;
    }

    /// mox_model__process
    fn mox_process(&self, sraw: f32) -> f32 {
        ((sraw - self.mox_sraw_mean) / -(self.mox_sraw_std + SRAW_STD_BONUS)) * INDEX_GAIN
    }

    /// mean_variance_estimator___calculate_gamma
    fn mve_calculate_gamma(&mut self) {
        let uptime_limit = FIX16_MAX - SAMPLING_INTERVAL;
        if self.mve_uptime_gamma < uptime_limit {
            self.mve_uptime_gamma += SAMPLING_INTERVAL;
        }
        if self.mve_uptime_gating < uptime_limit {
            self.mve_uptime_gating += SAMPLING_INTERVAL;
        }
        let sigmoid_gamma_mean = sigmoid(
            INIT_TRANSITION_MEAN,
            INIT_DURATION_MEAN,
            self.mve_uptime_gamma,
        );
        let gamma_mean = self.mve_gamma_mean
            + ((self.mve_gamma_initial_mean - self.mve_gamma_mean) * sigmoid_gamma_mean);
        let gating_threshold_mean = GATING_THRESHOLD
            + ((GATING_THRESHOLD_INITIAL - GATING_THRESHOLD)
                * sigmoid(
                    INIT_TRANSITION_MEAN,
                    INIT_DURATION_MEAN,
                    self.mve_uptime_gating,
                ));
        let sigmoid_gating_mean = sigmoid(
            GATING_THRESHOLD_TRANSITION,
            gating_threshold_mean,
            self.gas_index,
        );
        self.mve_gamma_mean_now = sigmoid_gating_mean * gamma_mean;

        let sigmoid_gamma_variance = sigmoid(
            INIT_TRANSITION_VARIANCE,
            INIT_DURATION_VARIANCE,
            self.mve_uptime_gamma,
        );
        let gamma_variance = self.mve_gamma_variance
            + ((self.mve_gamma_initial_variance - self.mve_gamma_variance)
                * (sigmoid_gamma_variance - sigmoid_gamma_mean));
        let gating_threshold_variance = GATING_THRESHOLD
            + ((GATING_THRESHOLD_INITIAL - GATING_THRESHOLD)
                * sigmoid(
                    INIT_TRANSITION_VARIANCE,
                    INIT_DURATION_VARIANCE,
                    self.mve_uptime_gating,
                ));
        let sigmoid_gating_variance = sigmoid(
            GATING_THRESHOLD_TRANSITION,
            gating_threshold_variance,
            self.gas_index,
        );
        self.mve_gamma_variance_now = sigmoid_gating_variance * gamma_variance;

        self.mve_gating_duration_minutes += (SAMPLING_INTERVAL / 60.0)
            * (((1.0 - sigmoid_gating_mean) * (1.0 + GATING_MAX_RATIO)) - GATING_MAX_RATIO);
        if self.mve_gating_duration_minutes < 0.0 {
            self.mve_gating_duration_minutes = 0.0;
        }
        if self.mve_gating_duration_minutes > GATING_MAX_DURATION_MINUTES {
            self.mve_uptime_gating = 0.0;
        }
    }

    /// mean_variance_estimator__process
    fn mve_process(&mut self, sraw: f32) {
        if !self.mve_initialized {
            self.mve_initialized = true;
            self.mve_sraw_offset = sraw;
            self.mve_mean = 0.0;
            return;
        }
        if self.mve_mean >= 100.0 || self.mve_mean <= -100.0 {
            self.mve_sraw_offset += self.mve_mean;
            self.mve_mean = 0.0;
        }
        let sraw = sraw - self.mve_sraw_offset;
        self.mve_calculate_gamma();
        let delta_sgp = (sraw - self.mve_mean) / GAMMA_SCALING;
        let c = if delta_sgp < 0.0 {
            self.mve_std - delta_sgp
        } else {
            self.mve_std + delta_sgp
        };
        let mut additional_scaling = 1.0;
        if c > 1440.0 {
            additional_scaling = (c / 1440.0) * (c / 1440.0);
        }
        self.mve_std = sqrtf(additional_scaling * (GAMMA_SCALING - self.mve_gamma_variance_now))
            * sqrtf(
                (self.mve_std * (self.mve_std / (GAMMA_SCALING * additional_scaling)))
                    + (((self.mve_gamma_variance_now * delta_sgp) / additional_scaling)
                        * delta_sgp),
            );
        self.mve_mean += (self.mve_gamma_mean_now * delta_sgp) / ADDITIONAL_GAMMA_MEAN_SCALING;
    }

    /// adaptive_lowpass__process
    fn lowpass(&mut self, sample: f32) -> f32 {
        if !self.lp_initialized {
            self.lp_x1 = sample;
            self.lp_x2 = sample;
            self.lp_x3 = sample;
            self.lp_initialized = true;
        }
        self.lp_x1 = ((1.0 - self.lp_a1) * self.lp_x1) + (self.lp_a1 * sample);
        self.lp_x2 = ((1.0 - self.lp_a2) * self.lp_x2) + (self.lp_a2 * sample);
        let mut abs_delta = self.lp_x1 - self.lp_x2;
        if abs_delta < 0.0 {
            abs_delta = -abs_delta;
        }
        let f1 = expf(LP_ALPHA * abs_delta);
        let tau_a = ((LP_TAU_SLOW - LP_TAU_FAST) * f1) + LP_TAU_FAST;
        let a3 = SAMPLING_INTERVAL / (SAMPLING_INTERVAL + tau_a);
        self.lp_x3 = ((1.0 - a3) * self.lp_x3) + (a3 * sample);
        self.lp_x3
    }
}

/// mean_variance_estimator___sigmoid__process
fn sigmoid(k: f32, x0: f32, sample: f32) -> f32 {
    let x = k * (sample - x0);
    if x < -50.0 {
        1.0
    } else if x > 50.0 {
        0.0
    } else {
        1.0 / (1.0 + expf(x))
    }
}

/// sigmoid_scaled__process, voc
fn sigmoid_scaled(sample: f32) -> f32 {
    let x = SIGMOID_K * (sample - SIGMOID_X0);
    if x < -50.0 {
        SIGMOID_L
    } else if x > 50.0 {
        0.0
    } else if sample >= 0.0 {
        let shift = (SIGMOID_L - (5.0 * INDEX_OFFSET)) / 4.0;
        ((SIGMOID_L + shift) / (1.0 + expf(x))) - shift
    } else {
        SIGMOID_L / (1.0 + expf(x))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_through_the_blackout_then_settles_near_100() {
        let mut voc = VocIndex::new();
        for _ in 0..=45 {
            assert_eq!(voc.process(30000), 0);
        }
        let mut index = 0;
        for _ in 0..600 {
            index = voc.process(30000);
        }
        assert!((95..=105).contains(&index), "steady air read {index}");
    }

    #[test]
    fn a_drop_in_the_raw_signal_raises_the_index() {
        // past fast initial learning
        let mut voc = VocIndex::new();
        for _ in 0..4000 {
            voc.process(30000);
        }
        // fewer ticks, more vocs
        let mut index = 0;
        for _ in 0..120 {
            index = voc.process(27000);
        }
        assert!(index > 200, "a VOC event read {index}");
    }
}
