
use crate::module::*;

pub struct Mult {
    in1: f64,
    in2: f64,
    out: f64,
}

impl Mult {
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
    fn advance(&mut self) {
        self.out = self.in1 * self.in2;
    }
}
