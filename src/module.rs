
use std::str::FromStr;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::Into;
use crate::units::Samples;

// Description of a terminals on a module
// XXX for now
pub type TerminalDescr = String;

// Modules need to be wrapped somehow because they are "dyn".
// Using reference counting simplifies storing modules in wires in a rack (but is not strictly necessary).
// Using RefCell lets us easily borrow the modules as mutable.
pub type ModRef = Rc<RefCell<dyn Module>>;
pub fn modref_new<T: 'static + Module>(data: T) -> ModRef { 
    Rc::new( RefCell::new(data) ) 
}

// proposed
#[allow(dead_code)]
pub struct TerminalDescr2 {
    name: String,
    min: f64,
    max: f64,
}

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

    // advance the clock by one sample, return false to request shutdown
    fn advance(&mut self) -> bool;

    // XXX some sort of interface for getting generic parameters
    // and setting them from ascii strings?

    fn set_named_input(&mut self, mod_name: &str, name: &str, val: f64) -> Result<(), String> {
        let (ins, _) = self.get_terminals();
        let idx = ins.iter().position(|n| n.eq(name))
                        .ok_or(format!("{} has no input named {}", mod_name, name))?;
        // XXX set_input should report errors
        Ok(self.set_input(idx, val))
    }

    fn get_named_output(&self, mod_name: &str, name: &str) -> Result<f64, String> {
        let (_, outs) = self.get_terminals();
        let idx = outs.iter().position(|n| n.eq(name))
                    .ok_or(format!("{} has no output named {}", mod_name, name))?;
        self.get_output(idx).ok_or(format!("can't set {}", name))
    }
}

// Connection from module inputs to module outputs
struct Wire {
    from_mod: ModRef,
    from_out: usize,

    to_mod_name: String,
    to_mod: ModRef,
    to_in: usize,
}

// a rack owns all of its modules and manages them
pub struct Rack {
    modules: HashMap<String, ModRef>,
    wires: Vec<Wire>,
    // XXX cache of output values for each module...
}

pub fn input_idx(m: &dyn Module, mod_name: &str, name: &str) -> Result<usize, String> {
    let (ins, _) = m.get_terminals();
    ins.iter().position(|n| n.eq(name))
            .ok_or(format!("{} has no input named {}", mod_name, name))
}

pub fn output_idx(m: &dyn Module, mod_name: &str, name: &str) -> Result<usize, String> {
    let (_, outs) = m.get_terminals();
    outs.iter().position(|n| n.eq(name))
            .ok_or(format!("{} has no output named {}", mod_name, name))
}

impl Rack {
    pub fn new() -> Self {
        Rack { 
            modules: HashMap::new(),
            wires: Vec::new(),
        }
    }

    // Add a module and its associated ID (name).
    pub fn add_module(&mut self, name: &str, m: ModRef) -> Result<(), String> {
        if self.modules.contains_key(name) {
            Err(format!("redefinition of {}", name))
        } else {
            self.modules.insert(name.to_owned(), m);
            Ok(())
        }
    }

    // Add a wire and return its index on success.
    pub fn add_wire(&mut self, 
                    from_mod_name: &str, from_out_name: &str,
                    to_mod_name: &str, to_in_name: &str) -> Result<(), String> {
        let from_mod = self.modules.get(from_mod_name).ok_or(format!("no module {}", from_mod_name))?;
        let to_mod = self.modules.get(to_mod_name).ok_or(format!("no module {}", to_mod_name))?;

        let out_idx = output_idx(&*from_mod.borrow(), from_mod_name, from_out_name)?;
        let in_idx = input_idx(&*to_mod.borrow(), to_mod_name, to_in_name)?;

        if self.wires.iter().any(|w| w.to_mod_name == to_mod_name && w.to_in == in_idx) {
            return Err(format!("{}'s {} input is already connected", to_mod_name, to_in_name));
        }

        let wire = Wire {
            from_mod: from_mod.to_owned(),
            from_out: out_idx,
            to_mod_name: to_mod_name.to_owned(),
            to_mod: to_mod.to_owned(),
            to_in: in_idx,
        };
        self.wires.push(wire);
        Ok(())
    }

    // Returns false if any module requests a shutdown
    pub fn advance(&mut self) -> bool {
        // advance the clock of all modules
        let mut keep_running = true;
        for (_, module) in self.modules.iter_mut() {
            let mut m = module.borrow_mut();
            let ok = m.advance();
            keep_running = keep_running && ok;
        }

        // Copy data across wires
        for w in self.wires.iter() {
            let out = w.from_mod.borrow().get_output(w.from_out).unwrap_or(0.0);
            w.to_mod.borrow_mut().set_input(w.to_in, out);
        }
        return keep_running;
    }

    pub fn run(&mut self, time: impl Into<Samples>) -> bool {
        let samples : Samples = time.into();
        for _ in 0 .. samples.0 {
            if !self.advance() {
                return false;
            }
        }
        return true;
    }

    pub fn set_input(&mut self, mod_name: &str, in_name: &str, val: f64) -> Result<(), String> {
        let m = self.modules.get(mod_name).ok_or(format!("no module named {}", mod_name))?;
        m.borrow_mut().set_named_input(mod_name, in_name, val)
    }
    pub fn get_output(&mut self, mod_name: &str, out_name: &str) -> Result<f64, String> {
        let m = self.modules.get(mod_name).ok_or(format!("no module named {}", mod_name))?;
        m.borrow().get_named_output(mod_name, out_name)
    }
}

pub fn parse<T: FromStr>(name: &str, val: &str) -> Result<T, String> {
    val.parse().map_err(|_| format!("can't parse {} '{}", name, val))
}

