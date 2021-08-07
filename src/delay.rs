
use crate::loader::Loader;
use crate::units::{Samples, Sec};
use crate::module::*;

const MINDEPTH: f64 = 0.01;

pub struct Delay {
    ring: Vec<f64>,
    dry: f64,
    fb: f64,
    rpos: usize, // invariant: less than ring.len()
    wpos: usize, // invariant: less than ring.len()

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

    pub fn new(maxdelay: impl Into<Samples>, dry: f64, fb: f64) -> Self {
        let Samples(maxd) = maxdelay.into();
        Self{
            ring: vec![0.0; maxd as usize],
            dry: dry,
            fb: fb,
            inp: 0.0,
            val: 0.0,
            rpos: 0,
            wpos: 0,
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
    pub fn set_delay(&mut self, v: impl Into<Samples>) {
        let Samples(delay) = v.into();
        
        // rpos lags wpos by delay samples.
        self.rpos = mod_sub(self.wpos, delay as usize, self.ring.len());
    }

    pub fn advance(&mut self) -> f64 {
        let delayed = self.ring[self.rpos] * self.fb;
        self.ring[self.wpos] = delayed + self.inp;

        self.rpos = mod_inc(self.rpos, self.ring.len());
        self.wpos = mod_inc(self.wpos, self.ring.len());

        self.val = delayed + self.dry * self.inp;
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
        return None;
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 { self.set_input(value); }

        // XXX show error if value is too large?  right now we silently wrap around
        if idx == 1 { self.set_delay(Sec(value)); }
    }

    fn advance(&mut self) -> bool {
        Delay::advance(self);
        return true;
    }
}

pub fn init(l: &mut Loader) {
    l.register("delay", Delay::from_cmd);
}
