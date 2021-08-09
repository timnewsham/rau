
use crate::additive::Function;
use crate::simple::Gen as Osc;
use crate::units::{RadPS, Hz};
use crate::module::*;

// first-order all-pass filter with non-constant phase response
pub struct AllPass {
    g: f64,
    delay1: f64,
    delay2: f64,
    inp: f64,
    val: f64,
}

impl AllPass {
    pub fn new(g: f64) -> Self {
        AllPass { g, delay1: 0.0, delay2: 0.0, inp: 0.0, val: 0.0 }
    }

    pub fn set_g(&mut self, g: f64) {
        self.g = g;
    }

    pub fn set_input(&mut self, v: f64) {
        self.inp = v;
    }

    // ref: https://s3.amazonaws.com/embeddedrelated/user/14446/fig.%203_88181.jpg
    pub fn advance(&mut self) -> f64 {
        self.val = self.delay1 + self.g * (self.inp - self.delay2);
        self.delay1 = self.inp;
        self.delay2 = self.val;
        self.val
    }
}

const MAXG: f64 = 0.95;

pub struct Phaser {
    lfo: Osc,
    f1: AllPass,
    f2: AllPass,
    f3: AllPass,
    f4: AllPass,
    width: f64,
    fb: f64,
    delay: f64,
    inp: f64,
    val: f64,
}

impl Phaser {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 5 {
            return Err(format!("usage: {} functype freq width feedback", args[0]));
        }
        let func = parse::<Function>("functype", args[1])?;
        let freq = parse::<f64>("freq", args[2])?;
        let width = parse::<f64>("width", args[3])?;
        let fb = parse::<f64>("feedback", args[4])?;

        // XXX sanity check width, fb
        Ok( modref_new(Self::new(func, Hz(freq), width, fb)) )
    }
    
    pub fn new(func: Function, freq: impl Into<RadPS>, width: f64, fb: f64) -> Self {
        let g4 = 0.9 * MAXG;
        Phaser {
            lfo: Osc::new(func, freq),
            f1: AllPass::new(g4 / 8.0),
            f2: AllPass::new(g4 / 4.0),
            f3: AllPass::new(g4 / 2.0),
            f4: AllPass::new(g4 / 1.0),
            width,
            fb,
            delay: 0.0,
            inp: 0.0,
            val: 0.0,
        }
    }

    pub fn set_freq(&mut self, freq: impl Into<RadPS>) {
        self.lfo.set_freq(freq);
    }

    pub fn set_input(&mut self, v: f64) {
        self.inp = v;
    }

    // ref: https://ccrma.stanford.edu/realsimple/DelayVar/Phasing_First_Order_Allpass_Filters.html
    pub fn advance(&mut self) -> f64 {
        let inp = self.inp + self.fb * self.delay;

        // g4 oscillate between -width and width which must be between -MAXG and MAXG
        let lfo = self.lfo.advance();
        let g4 = self.width * MAXG * lfo;
        assert!(-MAXG < g4 && g4 < MAXG);

        self.f1.set_g(g4 / 8.0);
        self.f2.set_g(g4 / 4.0);
        self.f3.set_g(g4 / 2.0);
        self.f4.set_g(g4 / 1.0);

        self.f1.set_input(inp);
        self.f2.set_input(self.f1.advance());
        self.f3.set_input(self.f2.advance());
        self.f4.set_input(self.f3.advance());
        let out = self.f4.advance();

        self.delay = out;
        self.val = out + self.inp;
        self.val
    }
}

impl Module for Phaser {
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        // XXX more inputs
        (vec!["in".to_string(),
              "freq".to_string()],
         vec!["out".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 { return Some(self.val); }
        None
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 { self.set_input(value); }
        if idx == 1 { self.set_freq(Hz(value)); }
    }

    fn advance(&mut self) -> bool {
        Phaser::advance(self);
        true
    }
}

pub fn init(l: &mut Loader) {
    l.register("phaser", Phaser::from_cmd);
}

