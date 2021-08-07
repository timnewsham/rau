
use crate::additive::Function;
use crate::simple::Gen as Osc;
use crate::delay::Delay;
use crate::units::{RadPS, Hz, Sec};
use crate::module::*;

pub struct Flange {
    delay: Delay,
    lfo: Osc,
    manual: f64, // 0 < manual < 1.0
    width: f64, // 0 < width < 1.0
    val: f64,
}

const MAXDELAY: f64 = 10e-3; // in seconds

impl Flange {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 7 {
            return Err(format!("usage: {} functype freq manual width dry feedback", args[0]));
        }
        let func = parse::<Function>("functype", args[1])?;
        let freq = parse::<f64>("freq", args[2])?;
        let manual = parse::<f64>("manual", args[3])?;
        let width = parse::<f64>("width", args[4])?;
        let dry = parse::<f64>("dry", args[5])?;
        let fb = parse::<f64>("feedback", args[6])?;

        // XXX sanity check manual, width, dry, fb
        Ok( modref_new(Self::new(func, Hz(freq), manual, width, dry, fb)) )
    }

    // manual is fraction of MAXDELAY
    // width is fraction, at 0.99, lfo sweeps full range from 0 to MAXDELAY (centered at manual)
    pub fn new(func: Function, freq: impl Into<RadPS>, manual: f64, width: f64, dry: f64, fb: f64) -> Self {
        assert!(0.0 < manual && manual < 1.0);
        assert!(0.0 < width && width < 1.0); // XXX allow negative width for inverting phase of lfo?

        let lfo = Osc::new(func, freq);
        let delay = Delay::new(Sec(MAXDELAY), dry, fb);
        Flange { 
            delay: delay,
            lfo: lfo,
            manual: manual,
            width: width,
            val: 0.0,
        }
    }

    pub fn set_freq(&mut self, freq: impl Into<RadPS>) {
        self.lfo.set_freq(freq);
    }

    pub fn set_manual(&mut self, manual: f64) {
        assert!(0.0 < manual && manual < 1.0);
        self.manual = manual;
    }

    pub fn set_width(&mut self, width: f64) {
        assert!(0.0 < width && width < 1.0);
        self.width = width;
    }

    pub fn set_fb(&mut self, fb: f64) {
        self.delay.set_fb(fb);
    }

    pub fn set_dry(&mut self, dry: f64) {
        self.delay.set_dry(dry);
    }

    pub fn set_input(&mut self, v: f64) {
        self.delay.set_input(v);
    }
    pub fn advance(&mut self) -> f64 {
        let m = self.manual * MAXDELAY; // in seconds
        let w = 0.5 * m * self.width; // in seconds
        let lfo = self.lfo.advance();
        let delay = m + lfo * w; // always between 0 and MAXDELAY
        self.delay.set_delay(Sec(delay));
        self.val = self.delay.advance();
        self.val
    }
}

impl Module for Flange {
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        // XXX more inputs
        (vec!["in".to_string(),
              "freq".to_string()],
         vec!["out".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 { return Some(self.val); }
        return None;
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 { self.set_input(value); }
        if idx == 1 { self.set_freq(Hz(value)); }
    }

    fn advance(&mut self) -> bool {
        Flange::advance(self);
        return true;
    }
}

pub fn init(l: &mut Loader) {
    l.register("flange", Flange::from_cmd);
}

