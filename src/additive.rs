
use std::str::FromStr;
use std::convert::Into;
use std::f64::consts::PI;
use crate::units::{RadPS, MAXRADPS, Hz, MAXHZ};
use crate::module::*;

#[derive(PartialEq, Copy, Clone)]
pub enum Function{ SIN, TRI, SAWUP, SAWDOWN, SQUARE }

impl Default for Function {
    fn default() -> Self { Function::SIN }
}

impl FromStr for Function {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "sin" { return Ok(Function::SIN); }
        if s == "tri" { return Ok(Function::TRI); }
        if s == "sawup" { return Ok(Function::SAWUP); }
        if s == "sawdown" { return Ok(Function::SAWDOWN); }
        if s == "square" { return Ok(Function::SQUARE); }
        return Err(format!("unrecognized function '{}'", s));
    }
}

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
    phase: f64, // in radians, invariant: 0 <= phase < 2*PI
    velocity: RadPS, // invariant: 0 <= velocity <= PI

    val: f64,
}

#[allow(dead_code)]
impl Gen {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 4 {
            return Err(format!("usage: {} functype freq order", args[0]));
        }
        let func = parse::<Function>("functype", args[1])?;
        let freq = parse::<f64>("freq", args[2])?;
        let order = parse::<usize>("order", args[3])?;
        Ok( modref_new(Self::new(func, Hz(freq), order)) )
    }

    pub fn new(typ: Function, freq: impl Into<RadPS>, n: usize) -> Self {
        Self {
            phase: 0.0,
            velocity: freq.into(),
            series: get_series(typ, n),
            val: 0.0
        }
    }

    pub fn set_freq(&mut self, freq: impl Into<RadPS>) {
        self.velocity = freq.into();
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

    pub fn advance(&mut self) -> f64 {
        self.phase = (self.phase + self.velocity.0) % (2.0 * PI);

        let mut x = 0.0;
        for param in self.series.iter() {
            if param.k as f64 * self.velocity.0 <= MAXRADPS { // disallow aliasing
                x += param.amp * (param.k as f64 * self.phase).sin();
            }
        }
        self.val = x;
        self.val
    }
}

impl Module for Gen {
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        (vec!["freq".to_string()],
         vec!["out".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 { return Some(self.val); }
        None
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 {
            self.velocity = Hz(value.clamp(0.0, MAXHZ)).into();
        }
    }

    fn advance(&mut self) -> bool {
        Gen::advance(self);
        return true;
    }
}

pub fn init(l: &mut Loader) {
    l.register("osc", Gen::from_cmd);
}
