
use std::convert::Into;
use crate::units::Samples;
use crate::module::*;

#[derive(Debug)]
enum EnvMode{ Attack, Decay, Release }

pub struct Envelope {
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,

    mode: EnvMode,
    val: f64,
    gate: bool,
    last_gate: bool,
}

// exponential decay factor to decay to 10% of starting value after t seconds.
fn decay_factor(time: impl Into<Samples>) -> f64 {
    // exp decay:
    // x[n+1] = r x[n],  x[0] = 1.0
    // x[n] = r^n 
    // log(x[n]) = n * log(r)
    // set x[N] be 0.1 (10dB down) of the starting value
    // -1 = N * log(r)
    // -1 = N * log(r)
    // -1/N = log(r)
    // r = 10^(-1/N)
    let Samples(n) =  time.into();
    assert!(n > 0);
    (10.0_f64).powf(-1.0 / (n as f64))
}

impl Envelope {
    // a,d,r in seconds
    // s as a level from 0..=1.0
    pub fn new(a: impl Into<Samples>, d: impl Into<Samples>, s: f64, r: impl Into<Samples>) -> Self {
        assert!(0.0 <= s && s <= 1.0);
        Envelope {
            attack: decay_factor(a),
            decay: decay_factor(d),
            sustain: s,
            release: decay_factor(r),
            mode: EnvMode::Release,
            val: 0.0,
            gate: false,
            last_gate: false,
        }
    }

    pub fn gen(&self) -> f64 { self.val }

    pub fn set_gate(&mut self, g: bool) {
        self.gate = g;
    }
}

impl Module for Envelope {
    fn advance(&mut self) {
        let last_gate = self.last_gate;
        self.last_gate = self.gate;
        if self.gate != last_gate {
            match self.gate {
                true => self.mode = EnvMode::Attack,
                false => self.mode = EnvMode::Release,
            };
        }

        match self.mode {
        EnvMode::Attack =>
            if self.val < 1.0 {
                self.val = 1.1 - (1.1 - self.val) * self.attack;
            } else {
                self.val = 1.0;
                self.mode = EnvMode::Decay;
            },
        EnvMode::Decay => 
            if self.val > self.sustain {
                self.val = self.sustain + (self.val - self.sustain) * self.decay;
            },
        EnvMode::Release => 
            self.val = self.val * self.release,
        };
    }

    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        (vec!["gate".to_string()],
         vec!["out".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 { Some(self.val) } else { None }
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 { self.set_gate(value >= 0.5) }
    }
}

