
use std::convert::Into;
use std::f64::consts::PI;
use crate::units::{RadPS, MAXRADPS};
use crate::gen;

#[derive(PartialEq, Copy, Clone)]
pub enum Function{ SIN, TRI, SAWUP, SAWDOWN, SQUARE }

// Param in a harmonic series
pub struct HarmonicParam {
    pub k: usize,
    pub amp: f64,
}

// shorthand for (-1)^k
fn powneg1(k: usize) -> f64 {
    (-1.0_f64).powf(k as f64)
}

fn get_series(func: Function, n: usize) -> Vec<HarmonicParam> {
    match func {
        Function::SIN => vec![HarmonicParam{ k: 1, amp: 1.0 }],
        Function::SAWUP => (1..=n).map(|k|
                HarmonicParam{ k: k, amp: -2.0 * powneg1(k) / (k as f64 * PI) }
            ).collect(),
        Function::SAWDOWN => (1..=n).map(|k|
                HarmonicParam{ k: k, amp: 2.0 * powneg1(k) / (k as f64 * PI) }
            ).collect(),
        Function::TRI => (1..=n).map(|nn| {
                let k = 2*nn - 1; // odd harmonics
                HarmonicParam{ k: k, amp: 8.0 * powneg1((k-1)/2) / (k as f64 * PI).powf(2.0) }
            }).collect(),
        Function::SQUARE => (1..=n).map(|nn| {
                let k = 2*nn - 1; // odd harmonics
                HarmonicParam{ k: k, amp: -4.0 * powneg1(k) / (k as f64 * PI) }
            }).collect(),
    }
}

// An additive generator generates a signal as a sum of SIN waves.
pub struct Gen {
    pub series: Vec<HarmonicParam>,

    // invariant: 0 <= phase < 2*PI
    phase: f64, // in radians

    // invariant: 0 <= velocity <= PI
    velocity: RadPS,
}

#[allow(dead_code)]
impl Gen {
    pub fn new(typ: Function, freq: impl Into<RadPS>, n: usize) -> Self {
        Self {
            phase: 0.0,
            velocity: freq.into(),
            series: get_series(typ, n),
        }
    }

    pub fn set_func(&mut self, typ: Function, n: usize) {
        self.series = get_series(typ, n);
    }

    pub fn set_phase(&mut self, theta: f64) {
        debug_assert!(theta >= 0.0);
        self.phase = theta % (2.0 * PI);
    }

    pub fn cost(&self) -> usize {
        let mut cost = 0;
        for param in self.series.iter() {
            if param.k as f64 * self.velocity.0 <= MAXRADPS {
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
            if param.k as f64 * self.velocity.0 <= MAXRADPS {
                x += param.amp * (param.k as f64 * self.phase).sin();
            }
        }
        x
    }

    fn cost(&self) -> usize {
        Gen::cost(self)
    }
}

