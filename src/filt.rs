
use std::str::FromStr;
use std::convert::Into;
use crate::units::{RadPS, Hz};
use crate::module::*;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum FiltType { LP, BP, Notch, HP, LowShelf, CenterShelf, HighShelf }

impl Default for FiltType {
    fn default() -> Self { FiltType::LP }
}

impl FromStr for FiltType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "lp" { return Ok(FiltType::LP); }
        if s == "bp" { return Ok(FiltType::BP); }
        if s == "notch" { return Ok(FiltType::Notch); }
        if s == "hp" { return Ok(FiltType::HP); }
        if s == "lowshelf" { return Ok(FiltType::LowShelf); }
        if s == "centershelf" { return Ok(FiltType::CenterShelf); }
        if s == "highshelf" { return Ok(FiltType::HighShelf); }
        return Err(format!("unrecognized filttype '{}'", s));
    }
}

#[derive(Default, Debug)]
pub struct Filter {
    typ: FiltType,
    freq: f64,
    gain: f64,
    q: f64,

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
        let RadPS(w) = freq.into();
        let mut v = Self { 
            typ,
            freq: w,
            gain,
            q,
            ..Default::default() 
        };
        v.recalc();
        v 
    }

    pub fn set_freq(&mut self, freq: impl Into<RadPS>) {
        let RadPS(w) = freq.into();
        self.freq = w;
        self.recalc();
    }

    fn recalc(&mut self) {
        // reference: https://webaudio.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html
        #[allow(non_snake_case)]
        let A = 10.0_f64.powf(self.gain/40.0);
        let w = self.freq;
        let q = self.q;

        let cw = w.cos();
        let sw = w.sin();
        let alpha = 0.5 * sw / q;
        match self.typ {
        FiltType::LowShelf => {
                let g = A.sqrt() * sw / q; // short name for: 2 sqrt(A) alpha
                let a0 = (A+1.0) + (A-1.0) * cw + g;

                self.b0 = A * ((A+1.0) - (A-1.0) * cw + g) / a0;
                self.b1 = 2.0 * A * ((A-1.0) - (A+1.0) * cw) / a0;
                self.b2 = A * ((A+1.0) - (A-1.0) * cw - g) / a0;
                self.a1 = -2.0 * ((A-1.0) + (A+1.0) * cw) / a0;
                self.a2 = ((A+1.0) + (A-1.0) * cw - g) / a0;
            },
        FiltType::HighShelf => {
                let g = A.sqrt() * sw / q; // short name for: 2 sqrt(A) alpha
                let a0 = (A+1.0) - (A-1.0) * cw + g;

                self.b0 = A * ((A+1.0) + (A-1.0) * cw + g) / a0;
                self.b1 = -2.0 * A * ((A-1.0) + (A+1.0) * cw) / a0;
                self.b2 = A * ((A+1.0) + (A-1.0) * cw - g) / a0;
                self.a1 = 2.0 * ((A-1.0) - (A+1.0) * cw) / a0;
                self.a2 = ((A+1.0) - (A-1.0) * cw - g) / a0;
            },
        FiltType::CenterShelf => {
                let a0 = 1.0 + alpha / A;

                self.b0 = (1.0 + alpha * A) / a0;
                self.b1 = (-2.0 * cw) / a0;
                self.b2 = (1.0 - alpha * A) / a0;
                self.a1 = (-2.0 * cw) / a0;
                self.a2 = (1.0 - alpha / A) / a0;
            },

        FiltType::LP => {
                let a0 = 1.0 + alpha / A;

                self.b0 = 0.5 * (1.0 - cw) / a0;
                self.b1 = (1.0 - cw) / a0;
                self.b2 = 0.5 * (1.0 - cw) / a0;
                self.a1 = -2.0 * cw / a0;
                self.a2 = (1.0 - alpha) / a0;
            },
        FiltType::BP => {
                let a0 = 1.0 + alpha / A;
 
                self.b0 = alpha / a0;
                self.b1 = 0.0 / a0;
                self.b2 = -alpha / a0;
                self.a1 = -2.0 * cw / a0;
                self.a2 = (1.0 - alpha) / a0;
            },
        FiltType::Notch => {
                let a0 = 1.0 + alpha / A;

                self.b0 = 1.0 / a0;
                self.b1 = -2.0 * cw / a0;
                self.b2 = 1.0 / a0;
                self.a1 = -2.0 * cw / a0;
                self.a2 = (1.0 - alpha) / a0;
            },
        FiltType::HP => {
                let a0 = 1.0 + alpha / A;

                self.b0 = 0.5 * (1.0 + cw) / a0;
                self.b1 = -1.0 * (1.0 + cw) / a0;
                self.b2 = 0.5 * (1.0 + cw) / a0;
                self.a1 = -2.0 * cw / a0;
                self.a2 = (1.0 - alpha) / a0;
            },
        };
    }
}

impl Module for Filter {
    // XXX terminals for freq, gain and Q and type?
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        (vec!["in".to_string(),
              "freq".to_string()],
         vec!["out".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 { Some(self.val) } else { None }
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 { self.inp = value; }
        if idx == 1 { self.set_freq(Hz(value)); }
    }

    fn advance(&mut self) -> bool {
        let delay0 = self.inp          - self.a1 * self.delay1 - self.a2 * self.delay2;
        self.val   = self.b0 * delay0  + self.b1 * self.delay1 + self.b2 * self.delay2;
        self.delay2 = self.delay1;
        self.delay1 = delay0;
        //println!("{:?}", self);
        true
    }
}

pub fn init(l: &mut Loader) {
    l.register("filter", Filter::from_cmd);
}

