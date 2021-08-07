
use std::convert::Into;
use crate::module;
use crate::units::{RadPS, Hz, MAXHZ};

// Interface for anything generating values on a per-sample basis.
pub trait Gen {
    fn advance(&mut self) -> bool;
    fn gen(&self) -> f64;
    fn set_freq(&mut self, freq: impl Into<RadPS>);

    // XXX prob shouldnt be here
    fn cost(&self) -> usize;
}

impl<T> module::Module for T where T: Gen {
    fn get_terminals(&self) -> (Vec<module::TerminalDescr>, Vec<module::TerminalDescr>) {
        (vec!["freq".to_string()],
         vec!["out".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 {
            // XXX get_output shouldnt compute, advance() should.
            Some(self.gen())
        } else {
            unreachable!();
        }
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 {
            let freq = Hz(value.clamp(0.0, MAXHZ));
            self.set_freq(freq);
        } else {
            unreachable!();
        }
    }

    fn advance(&mut self) -> bool {
        self.advance()
    }
}
