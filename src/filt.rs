
use std::convert::Into;
use crate::units::RadPS;
use crate::module::*;

pub enum FiltType { LP, LowShelf, BP, HighShelf, HP }

#[derive(Default, Debug)]
pub struct Filter {
    a1: f64,
    a2: f64,
    b0: f64,
    b1: f64,
    b2: f64,
    delay1: f64,
    delay2: f64,
    inp: f64,
    val: f64,
}

impl Filter {
    pub fn new(typ: FiltType, freq: impl Into<RadPS>, gain: f64, q: f64) -> Self {
        // reference: https://webaudio.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html
        #[allow(non_snake_case)]
        let A = 10.0_f64.powf(gain/40.0);
        let RadPS(w) = freq.into();
        let cw = w.cos();
        let sw = w.sin();
        let alpha = 0.5 * sw / q;
        match typ {
        FiltType::LowShelf => {
                let g = A.sqrt() * sw / q; // short name for: 2 sqrt(A) alpha
                let a0 = (A+1.0) + (A-1.0) * cw + g;
                Filter {
                    b0: A * ((A+1.0) - (A-1.0) * cw + g) / a0,
                    b1: 2.0 * A * ((A-1.0) - (A+1.0) * cw) / a0,
                    b2: A * ((A+1.0) - (A-1.0) * cw - g) / a0,
                    a1: -2.0 * ((A-1.0) + (A+1.0) * cw) / a0,
                    a2: ((A+1.0) + (A-1.0) * cw - g) / a0,
                    ..Default::default()
                }
            },
        FiltType::HighShelf => {
                let g = A.sqrt() * sw / q; // short name for: 2 sqrt(A) alpha
                let a0 = (A+1.0) - (A-1.0) * cw + g;
                Filter {
                    b0: A * ((A+1.0) + (A-1.0) * cw + g) / a0,
                    b1: -2.0 * A * ((A-1.0) + (A+1.0) * cw) / a0,
                    b2: A * ((A+1.0) + (A-1.0) * cw - g) / a0,
                    a1: 2.0 * ((A-1.0) - (A+1.0) * cw) / a0,
                    a2: ((A+1.0) - (A-1.0) * cw - g) / a0,
                    ..Default::default()
                }
            },
        FiltType::LP => {
                let a0 = 1.0 + alpha / A;
                Filter {
                    b0: 0.5 * (1.0 - cw) / a0,
                    b1: (1.0 - cw) / a0,
                    b2: 0.5 * (1.0 - cw) / a0,
                    a1: -2.0 * cw / a0,
                    a2: (1.0 - alpha) / a0,
                    ..Default::default()
                }
            },
        FiltType::HP => {
                let a0 = 1.0 + alpha / A;
                Filter {
                    b0: 0.5 * (1.0 + cw) / a0,
                    b1: -1.0 * (1.0 + cw) / a0,
                    b2: 0.5 * (1.0 + cw) / a0,
                    a1: -2.0 * cw / a0,
                    a2: (1.0 - alpha) / a0,
                    ..Default::default()
                }
            },
        FiltType::BP => {
                let a0 = 1.0 + alpha / A;
                Filter {
                    b0: (1.0 + alpha * A) / a0,
                    b1: (-2.0 * cw) / a0,
                    b2: (1.0 - alpha * A) / a0,
                    a1: (-2.0 * cw) / a0,
                    a2: (1.0 - alpha / A) / a0,
                    ..Default::default()
                }
            }
        }
    }
}

impl Module for Filter {
    // XXX terminals for freq, gain and Q and type?
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        (vec!["in".to_string()],
         vec!["out".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 { Some(self.val) } else { None }
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 {
            self.inp = value;
        }
    }

    fn advance(&mut self) {
        let delay0 = self.inp          - self.a1 * self.delay1 - self.a2 * self.delay2;
        self.val   = self.b0 * delay0  + self.b1 * self.delay1 + self.b2 * self.delay2;
        self.delay2 = self.delay1;
        self.delay1 = delay0;
        //println!("{:?}", self);
    }
}

