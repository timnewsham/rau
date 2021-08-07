
use crate::loader::Loader;
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

pub fn init(l: &mut Loader) {
    l.register("mult", Mult::from_cmd);
}
