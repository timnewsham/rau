
use std::convert::Into;
use std::f64::consts::PI;
use crate::units::{RadPS, MAXRADPS};
use crate::gen;
use crate::module;

// Param in a harmonic series
// note: k is usually but need not be an integer.
struct HarmonicParam {
    k: f64,
    amp: f64,
}

// Easier name for (-1)^k
fn neg1_k(k: f64) -> f64 {
    f64::powf(-1.0, k)
}

fn sine_series() -> Vec<HarmonicParam> {
    vec![HarmonicParam{ k: 1.0, amp: 1.0 }]
}

fn saw_up_series(n: u64) -> Vec<HarmonicParam> {
    debug_assert!(0 < n && n < 100);
    let mut weights = Vec::new();
    for k in 1..=n {
        let kf = k as f64;
        let w = -2.0 * neg1_k(kf) /(kf * PI);
        weights.push(HarmonicParam{ k: k as f64, amp: w });
    }
    weights
}

fn saw_down_series(n: u64) -> Vec<HarmonicParam> {
    let mut params = saw_up_series(n);
    for p in params.iter_mut() {
        p.amp *= -1.0;
    }
    params
}

fn triangle_series(n: u64) -> Vec<HarmonicParam> {
    let mut weights = Vec::new();
    for kk in 1..=n {
        let k = 2*kk - 1; // odd harmonics only
        let kf = k as f64;
        let w = 8.0 * neg1_k((kf-1.0)/2.0) / ((kf * PI).powf(2.0));
        weights.push(HarmonicParam{ k: k as f64, amp: w });
    }
    weights
}

fn square_series(n: u64) -> Vec<HarmonicParam> {
    debug_assert!(0 < n && n < 100);
    let mut weights = Vec::new();
    for kk in 1..=n {
        let k = 2*kk - 1; // odd harmonics only
        let kf = k as f64;
        let w = -4.0 * neg1_k(kf) / (kf * PI);
        weights.push(HarmonicParam{ k: k as f64, amp: w });
    }
    weights
}


// An additive generator generates a signal as a sum of SIN waves.
pub struct Gen {
    // invariant: 0 <= phase < 2*PI
    phase: f64, // in radians

    // invariant: 0 <= velocity < PI
    velocity: RadPS,

    series: Vec<HarmonicParam>,
}

#[allow(dead_code)]
impl Gen {
    // internal constructor
    fn new_series(freq: impl Into<RadPS>, series: Vec<HarmonicParam>) -> Self {
        // XXX truncate series to prevent aliasing
        Self {
            phase: 0.0,
            velocity: freq.into(),
            series: series,
        }
    }

    pub fn new_sine(freq: impl Into<RadPS>) -> Self {
        Self::new_series(freq, sine_series())
    }

    pub fn new_triangle(freq: impl Into<RadPS>, n: u64) -> Self {
        Self::new_series(freq, triangle_series(n))
    }

    pub fn new_saw_up(freq: impl Into<RadPS>, n: u64) -> Self {
        Self::new_series(freq, saw_up_series(n))
    }

    pub fn new_saw_down(freq: impl Into<RadPS>, n: u64) -> Self {
        Self::new_series(freq, saw_down_series(n))
    }

    pub fn new_square(freq: impl Into<RadPS>, n: u64) -> Self {
        Self::new_series(freq, square_series(n))
    }

    pub fn set_freq(&mut self, freq: impl Into<RadPS>) {
        self.velocity = freq.into();
    }

    pub fn set_phase(&mut self, theta: f64) {
        debug_assert!(theta >= 0.0);
        self.phase = theta % (2.0 * PI);
    }

    pub fn set_sine(&mut self) {
        self.series = sine_series();
    }

    pub fn set_triangle(&mut self, n: u64) {
        self.series = triangle_series(n);
    }

    pub fn set_saw_up(&mut self, n: u64) {
        self.series = saw_up_series(n);
    }

    pub fn set_saw_down(&mut self, n: u64) {
        self.series = saw_down_series(n);
    }

    pub fn set_square(&mut self, n: u64) {
        self.series = square_series(n);
    }

    pub fn cost(&self) -> usize {
        let mut cost = 0;
        for param in self.series.iter() {
            if param.k * self.velocity.0 < MAXRADPS {
                cost += 1;
            }
        }
        cost
    }
}

impl gen::Gen for Gen {
    fn advance(&mut self) {
        self.phase = (self.phase + self.velocity.0) % (2.0 * PI);
    }

    fn gen(&self) -> f64 {
        let mut x = 0.0;
        for param in self.series.iter() {
            // disallow aliasing
            // XXX would be better to trim series once instead of each gen
            if param.k * self.velocity.0 < MAXRADPS {
                x += param.amp * (param.k * self.phase).sin();
            }
        }
        x
    }

    fn cost(&self) -> usize {
        Gen::cost(self)
    }

}

