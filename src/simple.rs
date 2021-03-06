
use std::convert::Into;
use std::f64::consts::PI;
use crate::units::{RadPS, Hz, MAXHZ};
pub use crate::additive::Function;
use crate::module::*;

// Simple function wave shape generator
// Note: These will have low quality outputs at higher frequencies.
// but might be well suited for some LFO operations.
pub struct Gen {
    phase: f64, // in radians, invariant: 0 <= phase < 2*PI
    velocity: RadPS, // invariant: 0 <= velocity <= PI

    amp: f64,
    off: f64,
    func: fn(f64) -> f64,

    val: f64,
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
    } else if v <= 3.0 {
        2.0 - v             // (1.0 .. 3.0) -> (1.0 .. -1.0)
    } else {
        v - 4.0             // (3.0 .. 4.0) -> (-1.0 .. 0.0)
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
        if args.len() < 3 || args.len() > 5 {
            return Err(format!("usage: {} functype freq [amp off]", args[0]));
        }
        let func = parse::<Function>("functype", args[1])?;
        let freq = parse::<f64>("freq", args[2])?;
        let amp = if args.len() >= 3 { parse::<f64>("amp", args[3])? } else { 1.0 };
        let off = if args.len() >= 4 { parse::<f64>("off", args[4])? } else { 0.0 };

        Ok( modref_new(Self::new_full(func, Hz(freq), amp, off)) )
    }

    pub fn new_full(typ: Function, freq: impl Into<RadPS>, amp: f64, off: f64) -> Self {
        Self {
            phase: 0.0,
            velocity: freq.into(),
            amp,
            off,
            func: get_func(typ),
            val: 0.0,
        }
    }
    pub fn new(typ: Function, freq: impl Into<RadPS>) -> Self {
        Self::new_full(typ, freq, 1.0, 0.0)
    }

    // XXX pulse with pulse width parameter?

    pub fn set_freq(&mut self, freq: impl Into<RadPS>) {
        self.velocity = freq.into();
    }

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

    pub fn advance(&mut self) -> f64 {
        self.phase = (self.phase + self.velocity.0) % (2.0 * PI);
        self.val = self.off + self.amp * (self.func)(self.phase);
        self.val
    }
}

impl Module for Gen {
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        (vec!["freq".to_string()],
         vec!["out".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 { return Some(self.val) };
        None
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 {
            self.velocity = Hz(value.clamp(0.0, MAXHZ)).into();
        }
    }

    fn advance(&mut self) -> bool {
        Gen::advance(self);
        true
    }
}

pub fn init(l: &mut Loader) {
    l.register("osc2", Gen::from_cmd);
}
