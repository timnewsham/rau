
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::module::*;

type ParseFn = fn (&Vec<&str>) -> Result<ModRef, String>;
type RegMap = HashMap<&'static str, ParseFn>;
pub struct Loader {
    map: RegMap,
}

fn errstr(e: impl ToString) -> String {
    e.to_string()
}

fn parse_terminal<'a>(name: &str, s: &'a str) -> Result<(&'a str, &'a str), String> {
    let v: Vec<&str> = s.splitn(2, ":").collect();
    if v.len() != 2 {
        Err(format!("bad terminal format for wire {} '{}'", name, s))
    } else {
        Ok((v[0], v[1]))
    }
}

impl Loader {
    pub fn new() -> Self {
        Self{ map: HashMap::new() }
    }

    // Wish this could be done statically just once...
    pub fn init(&mut self) {
        crate::additive::init(self);
        crate::delay::init(self);
        crate::envelope::init(self);
        crate::file::init(self);
        crate::filt::init(self);
        crate::flange::init(self);
        crate::keyboard::init(self);
        crate::phaser::init(self);
        crate::simple::init(self);
        crate::speaker::init(self);
        crate::util::init(self);
    }

    pub fn register(&mut self, name: &'static str, f: ParseFn) {
        //println!("registered {}", name);
        self.map.insert(name, f);
    }

    fn proc_wire(&mut self, rack: &mut Rack, args: Vec<&str>) -> Result<(), String> {
        if args.len() != 3 {
            return Err(format!("wire needs two args"));
        }
        let (mod1,out) = parse_terminal("source", args[1])?;
        let (mod2,inp) = parse_terminal("dest", args[2])?;
        rack.add_wire(mod1, out, mod2, inp)
    }

    fn proc_mod(&mut self, name: &str, rack: &mut Rack, args: Vec<&str>) -> Result<(), String> {
        if args.len() == 0 {
            return Err(format!("module name without module definition"));
        }

        let newfunc = self.map.get(args[0]).ok_or(format!("unrecognized module '{}'", args[0]))?;
        let m = newfunc(&args)?;
        rack.add_module(name, m)
    }

    fn proc_line(&mut self, rack: &mut Rack, mut ws: Vec<&str>) -> Result<(), String> {
        if ws.len() == 0 {
            return Ok(());
        }

        if ws[0] == "wire" {
            return self.proc_wire(rack, ws);
        } else {
            let name = ws.remove(0).to_owned(); // ws.len() > 0
            return self.proc_mod(&name, rack, ws);
        }
    }

    pub fn load(&mut self, fname: &str) -> Result<Rack, String> {
        self.init();
        let file = File::open(fname).map_err(errstr)?;

        let mut rack = Rack::new();
        for (lno, line_or_err) in BufReader::new(file).lines().enumerate() {
            let line = line_or_err.map_err(|e| format!("{}: {}", fname, e))?;

            // strip comments
            let mut ws: Vec<&str> = line.split_whitespace().collect();
            if let Some(comment) = ws.iter().position(|s| s.starts_with("#")) {
                ws.resize(comment, ""); // strip comments
            }

            self.proc_line(&mut rack, ws).map_err(|e| format!("{}:{}: {}", fname, lno+1, e))?;
        }
        Ok(rack)
    }
}
