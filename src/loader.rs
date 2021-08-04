
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
//use std::path::Path;
use crate::module::*;

type ParseFn = fn (&Vec<&str>) -> Result<Box<dyn Module>, &'static str>;
type RegMap = HashMap<&'static str, ParseFn>;
pub struct Loader {
    map: RegMap
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

    fn wire_from_cmd(&self, _args: &Vec<&str>) -> Result<(), &'static str> {
        Err("not yet")
    }
    
    fn proc_line(&self, fname: &str, lno: usize, ws: &Vec<&str>) {
        //println!("{} words: {:?}", lno+1, ws);
        if ws[0] == "wire" {
            match self.wire_from_cmd(ws) {
                Err(e) => println!("{}:{}: {}", fname, lno, e),
                Ok(_) => (), //XXX do stuff!
            };
            return;
        } 

        if let Some(func) = self.map.get(ws[0]) {
            match func(ws) {
                Err(e) => println!("{}:{}: {}", fname, lno, e),
                Ok(_) => println!("success for {:?}", ws), //XXX do stuff!
            };
            return;
        }

        println!("{}:{}: unrecognized module", fname, lno);
        // ERR
    }

    pub fn load(&mut self, fname: &str) -> Result<(), String> {
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
    
            self.proc_line(fname, lno + 1, &ws);
        }
        Ok(())
    }
}
