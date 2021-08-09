
use crate::loader::Loader;
use crate::units::{FracSamples, Sec};
use crate::module::*;

const MINDEPTH: f64 = 1e-3; // at 48khz this is 48 samples

pub struct Delay {
    ring: Vec<f64>,
    dry: f64,
    fb: f64,
    rpos: usize, // invariant: less than ring.len()
    wpos: usize, // invariant: less than ring.len()
    interp: f64, // fractional sample past rpos

    inp: f64,
    val: f64,
}

impl Delay {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 4 {
            return Err(format!("usage: {} depth dry feedback", args[0]));
        }
        let mut depth = parse::<f64>("depth", args[1])?;
        let dry = parse::<f64>("dry", args[2])?;
        let fb = parse::<f64>("feedback", args[3])?;

        if depth < MINDEPTH {
            depth = MINDEPTH;
        }
        // XXX sanity check dry and fb
        Ok( modref_new(Self::new(Sec(depth), dry, fb)) )
    }

    pub fn new(maxdelay: impl Into<FracSamples>, dry: f64, fb: f64) -> Self {
        let FracSamples(maxd,_) = maxdelay.into();
        Self{
            ring: vec![0.0; maxd + 2],
            dry,
            fb,
            inp: 0.0,
            val: 0.0,
            rpos: 1,
            wpos: 0,
            interp: 0.0,
        }
    }

    pub fn set_fb(&mut self, v: f64) {
        self.fb = v;
    }
    pub fn set_input(&mut self, v: f64) {
        self.inp = v;
    }
    pub fn set_dry(&mut self, v: f64) {
        self.dry = v;
    }
    pub fn set_delay(&mut self, v: impl Into<FracSamples>) {
        let FracSamples(delay, frac_delay) = v.into();
        assert!(delay + 1 <= self.ring.len());
        assert!(0.0 <= frac_delay && frac_delay < 1.0);
        
        // rpos lags wpos by delay samples.
        self.rpos = mod_sub(self.wpos, delay, self.ring.len());
        self.interp = frac_delay;
    }

    pub fn advance(&mut self) -> f64 {
        // read delayed value with linear interpolation for fractional sample delay
        assert!(self.rpos != self.wpos); // must be at least 1 behind, next_rpos can be wpos.
        let next_rpos = mod_inc(self.rpos, self.ring.len());
        /* XXX ugh, this sounds horrible! what did I do wrong?
        let delayed = self.ring[self.rpos] * (1.0 - self.interp) +
                      self.ring[next_rpos] * self.interp;
        */
        let delayed = self.ring[self.rpos];

        // write new value into delay line
        let fb = self.fb * delayed;
        self.ring[self.wpos] = fb + self.inp;

        self.rpos = next_rpos;
        self.wpos = mod_inc(self.wpos, self.ring.len());

        self.val = fb + self.dry * self.inp;
        self.val
    }
}

impl Module for Delay {
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        (vec!["in".to_string(), "delay".to_string()],
         vec!["out".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 { return Some(self.val); }
        None
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 { self.set_input(value); }

        // XXX show error if value is too large?  right now we silently wrap around
        if idx == 1 { self.set_delay(Sec(value)); }
    }

    fn advance(&mut self) -> bool {
        Delay::advance(self);
        true
    }
}

pub fn init(l: &mut Loader) {
    l.register("delay", Delay::from_cmd);
}
