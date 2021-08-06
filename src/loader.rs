
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
//use std::path::Path;
use crate::module::*;

type ParseFn = fn (&Vec<&str>) -> Result<Box<dyn Module>, &'static str>;
type RegMap = HashMap<&'static str, ParseFn>;
pub struct Loader {
    map: RegMap,
}

fn errstr(e: impl ToString) -> String {
    e.to_string()
}

impl Loader {
    pub fn new() -> Self {
        Self{ map: HashMap::new() }
    }

    pub fn init(&mut self) {
        crate::additive::init(self);
        crate::envelope::init(self);
        crate::file::init(self);
        crate::filt::init(self);
        crate::keyboard::init(self);
        crate::simple::init(self);
        crate::speaker::init(self);
        crate::util::init(self);
    }

    pub fn register(&mut self, name: &'static str, f: ParseFn) {
        //println!("registered {}", name);
        self.map.insert(name, f);
    }

    fn wire_from_cmd(&self, args: Vec<&str>,
            fname: &str, lno: usize, 
            modtab: &mut HashMap<String, usize>,
            rack: &mut Rack) -> bool {
        if args.len() != 3 {
            println!("{}:{}: wire needs two args", fname, lno);
            return false;
        }
        let term1: Vec<&str> = args[1].splitn(2, ":").collect();
        let term2: Vec<&str> = args[2].splitn(2, ":").collect();
        println!("term1 {:?} term2 {:?}", term1, term2);
        if term1.len() != 2 || term2.len() != 2 {
            println!("{}:{}: bad module terminal format", fname, lno);
            return false;
        }
        if let (Some(mod1), Some(mod2)) = (modtab.get(term1[0]), modtab.get(term2[0])) {
            if rack.add_wire(*mod1, term1[1], *mod2, term2[1]).is_none() {
                println!("{}:{}: bad terminal name", fname, lno);
                return false;
            }
            return true;
        } else {
            println!("{}:{}: bad module name", fname, lno);
            return false;
        }
    }
    
    fn proc_line(&mut self, fname: &str, lno: usize, mut ws: Vec<&str>,
            modtab: &mut HashMap<String, usize>,
            rack: &mut Rack) -> bool {
        assert!(ws.len() > 0); // by construction

        //println!("{} words: {:?}", lno+1, ws);
        if ws[0] == "wire" {
            return self.wire_from_cmd(ws, fname, lno, modtab, rack);
        } 

        let name = ws.remove(0).to_owned(); // safe by construction
        if modtab.contains_key(&name) {
            println!("{}:{}: redefining name {}", fname, lno, name);
            return false;
        }
        if ws.len() == 0 {
            println!("{}:{}: module name without module definition", fname, lno);
            return false;
        }

        if let Some(func) = self.map.get(ws[0]) {
            match func(&ws) {
                Err(e) => {
                    println!("{}:{}: {}", fname, lno, e);
                    return false;
                    },
                Ok(module) => {
                    // XXX mod_id should have its own type for safety
                    let mod_id = rack.add_module(module);
                    modtab.insert(name.to_owned(), mod_id);
                    println!("success for {:?}", ws); //XXX do stuff!
                    },
            };
            return true;
        }

        println!("{}:{}: unrecognized module '{}'", fname, lno, ws[0]);
        return false;
    }

    pub fn load(&mut self, fname: &str) -> Result<Rack, String> {
        // XXX consider includign modmap in rack
        let mut rack = Rack::new();
        let mut modtab: HashMap<String, usize> = HashMap::new();

        self.init();
        let file = File::open(fname).map_err(errstr)?;
        for (lno, line_or_err) in BufReader::new(file).lines().enumerate() {
            let line = line_or_err.map_err(errstr)?;
            let mut ws: Vec<&str> = line.split_whitespace().collect();
            if let Some(comment) = ws.iter().position(|s| s.starts_with("#")) {
                ws.resize(comment, ""); // strip comments
            }
            if ws.len() == 0 { // skip empty lines
                continue;
            }
    
            if !self.proc_line(fname, lno + 1, ws, &mut modtab, &mut rack) { break }
        }
        Ok(rack)
    }
}
