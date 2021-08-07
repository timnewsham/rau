
use std::str::FromStr;
use std::convert::Into;
use crate::units::{RadPS, Hz};
use crate::module::*;
use crate::loader::Loader;

#[derive(PartialEq, Copy, Clone)]
pub enum FiltType { LP, LowShelf, BP, HighShelf, HP }

impl FromStr for FiltType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "lp" { return Ok(FiltType::LP); }
        if s == "lowshelf" { return Ok(FiltType::LowShelf); }
        if s == "bp" { return Ok(FiltType::BP); }
        if s == "highshelf" { return Ok(FiltType::HighShelf); }
        if s == "hp" { return Ok(FiltType::HP); }
        return Err(format!("unrecognized filttype '{}'", s));
    }
}

#[derive(Default, Debug)]
pub struct Filter {
    pub a1: f64,
    pub a2: f64,
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
    delay1: f64,
    delay2: f64,
    inp: f64,
    val: f64,
}

impl Filter {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 5 {
            return Err(format!("usage: {} filttype freq gain q", args[0]));
        }
        let typ = parse::<FiltType>("filttype", args[1])?;
        let f = parse::<f64>("freq", args[2])?;
        let g = parse::<f64>("gain", args[3])?;
        let q = parse::<f64>("q", args[4])?;
        Ok( modref_new(Self::new(typ, Hz(f), g, q)) )
    }

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

    fn advance(&mut self) -> bool {
        let delay0 = self.inp          - self.a1 * self.delay1 - self.a2 * self.delay2;
        self.val   = self.b0 * delay0  + self.b1 * self.delay1 + self.b2 * self.delay2;
        self.delay2 = self.delay1;
        self.delay1 = delay0;
        //println!("{:?}", self);
        return true;
    }
}

pub fn init(l: &mut Loader) {
    l.register("filter", Filter::from_cmd);
}

