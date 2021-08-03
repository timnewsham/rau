
use std::convert::Into;
use std::f64::consts::PI;
use crate::units::{RadPS, MAXRADPS};
use crate::gen;

#[derive(PartialEq, Copy, Clone)]
pub enum Function{ SIN, TRI, SAWUP, SAWDOWN, SQUARE }

// Param in a harmonic series
// note: k is usually but need not be an integer.
pub struct HarmonicParam {
    pub k: f64,
    pub amp: f64,
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
    pub series: Vec<HarmonicParam>,

    // invariant: 0 <= phase < 2*PI
    phase: f64, // in radians

    // invariant: 0 <= velocity <= PI
    velocity: RadPS,
}

fn get_series(typ: Function, n: u64) -> Vec<HarmonicParam> {
    match typ {
        Function::SIN => sine_series(),
        Function::TRI => triangle_series(n),
        Function::SAWUP => saw_up_series(n),
        Function::SAWDOWN => saw_down_series(n),
        Function::SQUARE => square_series(n),
    }
}

#[allow(dead_code)]
impl Gen {
    pub fn new(typ: Function, freq: impl Into<RadPS>, n: u64) -> Self {
        Self {
            phase: 0.0,
            velocity: freq.into(),
            series: get_series(typ, n),
        }
    }

    pub fn set_func(&mut self, typ: Function, n: u64) {
        self.series = get_series(typ, n);
    }

    pub fn set_phase(&mut self, theta: f64) {
        debug_assert!(theta >= 0.0);
        self.phase = theta % (2.0 * PI);
    }

    pub fn cost(&self) -> usize {
        let mut cost = 0;
        for param in self.series.iter() {
            if param.k * self.velocity.0 <= MAXRADPS {
                cost += 1;
            }
        }
        cost
    }
}

impl gen::Gen for Gen {
    fn set_freq(&mut self, freq: impl Into<RadPS>) {
        self.velocity = freq.into();
    }

    fn advance(&mut self) {
        self.phase = (self.phase + self.velocity.0) % (2.0 * PI);
    }

    fn gen(&self) -> f64 {
        let mut x = 0.0;
        for param in self.series.iter() {
            // disallow aliasing
            // XXX would be better to trim series once instead of each gen
            if param.k * self.velocity.0 <= MAXRADPS {
                x += param.amp * (param.k * self.phase).sin();
            }
        }
        x
    }

    fn cost(&self) -> usize {
        Gen::cost(self)
    }
}

