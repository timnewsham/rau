
use std::convert::Into;
use std::f64::consts::PI;
use crate::units::{RadPS};
use crate::gen;

// Simple function wave shape generator
pub struct Gen {
    // invariant: 0 <= phase < 2*PI
    phase: f64, // in radians

    // invariant: 0 <= velocity < PI
    velocity: RadPS,

    func: fn(f64) -> f64,
}

fn sine(phase: f64) -> f64 {
    phase.sin()
}

fn saw_up(phase: f64) -> f64 {
    let v = phase / PI;
    if v > 1.0 { v - 2.0 } else { v }
}

fn square(phase: f64) -> f64 {
    if phase < PI { 1.0 } else { -1.0 }
}

fn triangle(phase: f64) -> f64 {
    //debug_assert!(0.0 <= phase && phase <= (2.0 * PI));
    let v = phase * (2.0 / PI); // (0 .. 2PI) -> (0.0 .. 4.0)
    //debug_assert!(0.0 <= v && v <= 4.0);

    if v < 1.0 {
        v                   // (0 .. 1.0) -> (0 .. 1.0)
    } else {
        if v < 3.0 {
            2.0 - v         // (1.0 .. 3.0) -> (1.0 .. -1.0)
        } else {
            v - 4.0         // (3.0 .. 4.0) -> (-1.0 .. 0.0)
        }
    }
}

#[allow(dead_code)]
impl Gen {
    // internal constructor
    fn new_with_func(freq: impl Into<RadPS>, func: fn (f64) -> f64) -> Self {
        // XXX truncate series to prevent aliasing
        Self {
            phase: 0.0,
            velocity: freq.into(),
            func: func,
        }
    }

    pub fn new_sine(freq: impl Into<RadPS>) -> Self {
        Self::new_with_func(freq, sine)
    }

    pub fn new_saw_up(freq: impl Into<RadPS>) -> Self {
        Self::new_with_func(freq, saw_up)
    }

    pub fn new_triangle(freq: impl Into<RadPS>) -> Self {
        Self::new_with_func(freq, triangle)
    }

    pub fn new_square(freq: impl Into<RadPS>) -> Self {
        Self::new_with_func(freq, square)
    }

    // XXX pulse with pulse width parameter?

    pub fn set_freq(&mut self, freq: impl Into<RadPS>) {
        self.velocity = freq.into();
    }

    pub fn set_phase(&mut self, theta: f64) {
        debug_assert!(theta >= 0.0);
        self.phase = theta % (2.0 * PI);
    }

    pub fn set_sine(&mut self) {
        self.func = sine;
    }

    pub fn set_saw_up(&mut self) {
        self.func = saw_up;
    }

    pub fn set_triangle(&mut self) {
        self.func = triangle;
    }

    pub fn set_square(&mut self) {
        self.func = square;
    }
}

impl gen::Gen for Gen {
    fn advance(&mut self) {
        self.phase = (self.phase + self.velocity.0) % (2.0 * PI);
    }

    fn gen(&self) -> f64 {
        (self.func)(self.phase)
    }

    fn cost(&self) -> usize {
        1
    }
}

