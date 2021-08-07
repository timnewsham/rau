
use crate::module::*;

pub struct Mult {
    in1: f64,
    in2: f64,
    out: f64,
}

impl Mult {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 1 {
            return Err(format!("usage: {}", args[0]));
        }
        Ok( modref_new(Self::new()) )
    }

    pub fn new() -> Self {
        Self{ in1: 0.0, in2: 0.0, out: 0.0 }
    }
}

impl Module for Mult {
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        (vec!["in1".to_string(), "in2".to_string()],
         vec!["out".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 { Some(self.out) } else { None }
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 { self.in1 = value; }
        if idx == 1 { self.in2 = value; }
    }
    fn advance(&mut self) -> bool {
        self.out = self.in1 * self.in2;
        return true;
    }
}

pub struct Add {
    in1: f64,
    in2: f64,
    out: f64,
}

impl Add {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 1 {
            return Err(format!("usage: {}", args[0]));
        }
        Ok( modref_new(Self::new()) )
    }

    pub fn new() -> Self {
        Self{ in1: 0.0, in2: 0.0, out: 0.0 }
    }
}

impl Module for Add {
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        (vec!["in1".to_string(), "in2".to_string()],
         vec!["out".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 { Some(self.out) } else { None }
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 { self.in1 = value; }
        if idx == 1 { self.in2 = value; }
    }
    fn advance(&mut self) -> bool {
        self.out = self.in1 + self.in2;
        return true;
    }
}

pub struct Inv {
    inp: f64,
    out: f64,
}

impl Inv {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 1 {
            return Err(format!("usage: {}", args[0]));
        }
        Ok( modref_new(Self::new()) )
    }

    pub fn new() -> Self {
        Self{ inp: 0.0, out: 0.0 }
    }
}

impl Module for Inv {
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        (vec!["in".to_string()],
         vec!["out".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 { Some(self.out) } else { None }
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 { self.inp = value; }
    }
    fn advance(&mut self) -> bool {
        self.out = -self.inp;
        return true;
    }
}

pub struct Const {
    out: f64,
}

impl Const {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 2 {
            return Err(format!("usage: {} val", args[0]));
        }
        let v = parse::<f64>("value", args[1])?;
        Ok( modref_new(Self::new(v)) )
    }

    pub fn new(v: f64) -> Self {
        Self{ out: v }
    }
}

impl Module for Const {
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        (vec![],
         vec!["out".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 { Some(self.out) } else { None }
    }

    fn set_input(&mut self, _idx: usize, _value: f64) {
    }
    fn advance(&mut self) -> bool {
        // constant value is already set
        return true;
    }
}

// bias an output by scaling it and adding an offset
// Beware: If width is greater than off, then this will do strange things to an oscillator signal.
// But widths greather than off are fine for envelopes.
pub struct Bias {
    off: f64,
    width: f64,
    inp: f64,
    val: f64,
}

impl Bias {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 3 {
            return Err(format!("usage: {} off width", args[0]));
        }
        let off = parse::<f64>("freq", args[1])?;
        let width = parse::<f64>("freq", args[2])?;
        // XXX sanity check off and width
        Ok( modref_new(Self::new(off, width)) )
    }

    pub fn new(off: f64, width: f64) -> Self {
        Bias {
            off: off,
            width: width,
            inp: 0.0,
            val: 0.0,
        }
    }
}

impl Module for Bias {
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        (vec!["in".to_string()],
         vec!["out".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 { Some(self.val) } else { None }
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 { self.inp = value; }
    }

    fn advance(&mut self) -> bool {
        self.val = self.off + self.width * self.inp;
        return true;
    }
}

pub fn init(l: &mut Loader) {
    l.register("mult", Mult::from_cmd);
    l.register("add", Add::from_cmd);
    l.register("inv", Inv::from_cmd);
    l.register("const", Const::from_cmd);
    l.register("bias", Bias::from_cmd);
}
