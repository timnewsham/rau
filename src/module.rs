
use std::collections::HashMap;
use std::convert::Into;
use crate::units::Samples;

// Description of a terminals on a module
// XXX for now
pub type TerminalDescr = String;

// proposed
#[allow(dead_code)]
pub struct TerminalDescr2 {
    name: String,
    min: f64,
    max: f64,
}

// XXX TODO: quit signal global to the rack, that modules can connect to?
// XXX TODO: better typing of module IDs
// XXX TODO: include a map of module names to modules in the rack?

/*
 * Interface to modules with inputs and outputs and a sample-based clock.
 */
pub trait Module {
    // Get list of input and output terminals
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>);

    // Get a value from a terminal at output terminal idx
    // XXX we can guarantee Option is never None.
    fn get_output(&self, idx: usize) -> Option<f64>;

    // Set a value at a terminal at input terminal idx
    fn set_input(&mut self, idx: usize, value: f64);

    // advance the clock by one sample
    fn advance(&mut self);

    // XXX some sort of interface for getting generic parameters
    // and setting them from ascii strings?
}

// Connection from module inputs to module outputs
struct Wire {
    from_mod: usize,
    from_out: usize,
    to_mod: usize,
    to_in: usize,
}

// a rack owns all of its modules and manages them
pub struct Rack {
    modules: Vec<Box<dyn Module>>,
    wires: Vec<Wire>,
    // XXX cache of output values for each module...
}

impl Rack {
    pub fn new() -> Self {
        Rack { 
            modules: Vec::new(),
            wires: Vec::new(),
        }
    }

    // Add a module and return its index
    pub fn add_module(&mut self, m: Box<dyn Module>) -> usize {
        self.modules.push(m);
        self.modules.len() - 1
    }

    // Add a wire and return its index on success.
    pub fn add_wire(&mut self, 
                    from_mod_idx: usize, from_out: &str,
                    to_mod_idx: usize, to_in: &str) -> Option<usize> {
        let from_mod = self.modules.get(from_mod_idx)?;
        let (_, outs) = from_mod.get_terminals();
        let to_mod = &(*self.modules.get(to_mod_idx)?);
        let (ins, _) = to_mod.get_terminals();

        // XXX is this copying each string before comparing?
        // how can I get refs on each of the strings instead?
        let out_idx = outs.iter().position(|name| name.eq(from_out))?;
        let in_idx = ins.iter().position(|name| name.eq(to_in))?;

        // XXX check if output is already connected
        // right now conflicting wires will set_output multiple times

        let wire = Wire {
            from_mod: from_mod_idx,
            from_out: out_idx,
            to_mod: to_mod_idx,
            to_in: in_idx,
        };
        self.wires.push(wire);
        Some( self.wires.len() - 1 )
    }

    pub fn advance(&mut self) {
        // advance the clock of all modules
        for module in self.modules.iter_mut() {
            module.advance();
        }

        // Copy data across wires
        let mut out_cache = HashMap::new();
        for w in self.wires.iter() {
            let k = (w.from_mod, w.from_out);
            if let Some(out) = out_cache.get(&k) {
                self.modules[w.to_mod].set_input(w.to_in, *out);
            } else {
                let out = self.modules[w.from_mod].get_output(w.from_out).unwrap_or(0.0);
                self.modules[w.to_mod].set_input(w.to_in, out);
                out_cache.insert(k, out);
            }
        }
    }

    pub fn run(&mut self, time: impl Into<Samples>) {
        let samples : Samples = time.into();
        for _ in 0 .. samples.0 {
            self.advance();
        }
    }

    pub fn set_input(&mut self, mod_idx: usize, in_idx: usize, val: f64) -> Option<()> {
        if mod_idx >= self.modules.len() {
            return None;
        }
        self.modules[mod_idx].set_input(in_idx, val);
        Some(())
    }
    pub fn get_output(&mut self, mod_idx: usize, out_idx: usize) -> Option<f64> {
        if mod_idx >= self.modules.len() {
            return None;
        }
        self.modules[mod_idx].get_output(out_idx)
    }
}

