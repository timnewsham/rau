
use std::convert::Into;
use std::f64::consts::PI;
use crate::units::{RadPS, Hz};
use crate::gen;
use crate::loader::Loader;
pub use crate::additive::Function;
use crate::module::*;

// Simple function wave shape generator
// Note: These will have low quality outputs at higher frequencies.
// but might be well suited for some LFO operations.
pub struct Gen {
    // invariant: 0 <= phase < 2*PI
    phase: f64, // in radians

    // invariant: 0 <= velocity <= PI
    velocity: RadPS,

    amp: f64,
    off: f64,

    func: fn(f64) -> f64,
}

// XXX exponential ramp-up?  exponential decay?
// one-sided square waves?
// variable width pulses?

fn sine(phase: f64) -> f64 {
    phase.sin()
}

fn saw_up(phase: f64) -> f64 {
    let v = phase / PI;
    if v <= 1.0 { v } else { v - 2.0 }
}

fn saw_down(phase: f64) -> f64 {
    -saw_up(phase)
}

fn square(phase: f64) -> f64 {
    if phase <= PI { 1.0 } else { -1.0 }
}

fn triangle(phase: f64) -> f64 {
    //debug_assert!(0.0 <= phase && phase <= (2.0 * PI));
    let v = phase * (2.0 / PI); // (0 .. 2PI) -> (0.0 .. 4.0)
    //debug_assert!(0.0 <= v && v <= 4.0);

    if v <= 1.0 {
        v                   // (0 .. 1.0) -> (0 .. 1.0)
    } else {
        if v <= 3.0 {
            2.0 - v         // (1.0 .. 3.0) -> (1.0 .. -1.0)
        } else {
            v - 4.0         // (3.0 .. 4.0) -> (-1.0 .. 0.0)
        }
    }
}

fn get_func(typ: Function) -> fn(f64)->f64 {
    match typ {
        Function::SIN => sine,
        Function::TRI => triangle,
        Function::SAWUP => saw_up,
        Function::SAWDOWN => saw_down,
        Function::SQUARE => square,
    }
}

#[allow(dead_code)]
impl Gen {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 3 {
            return Err(format!("usage: {} functype freq", args[0]));
        }
        let func = parse::<Function>("functype", args[1])?;
        let freq = parse::<f64>("freq", args[2])?;
        Ok( modref_new(Self::new(func, Hz(freq))) )
    }

    // internal constructor
    pub fn new(typ: Function, freq: impl Into<RadPS>) -> Self {
        Self {
            phase: 0.0,
            velocity: freq.into(),
            amp: 1.0,
            off: 0.0,
            func: get_func(typ),
        }
    }

    // XXX pulse with pulse width parameter?

    pub fn set_phase(&mut self, theta: f64) {
        debug_assert!(theta >= 0.0);
        self.phase = theta % (2.0 * PI);
    }

    pub fn set_off(&mut self, off: f64) {
        self.off = off;
    }
    pub fn set_amp(&mut self, amp: f64) {
        self.amp = amp;
    }

    pub fn set_func(&mut self, typ: Function) {
        self.func = get_func(typ);
    }
}

impl gen::Gen for Gen {
    fn set_freq(&mut self, freq: impl Into<RadPS>) {
        self.velocity = freq.into();
    }

    fn advance(&mut self) -> bool {
        self.phase = (self.phase + self.velocity.0) % (2.0 * PI);
        return true;
    }

    fn gen(&self) -> f64 {
        self.off + self.amp * (self.func)(self.phase)
    }

    fn cost(&self) -> usize {
        1
    }
}

pub fn init(l: &mut Loader) {
    l.register("osc2", Gen::from_cmd);
}
