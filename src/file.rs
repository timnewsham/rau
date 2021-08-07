
use std::fs::File;
use std::io::{Write,BufWriter};
use std::convert::Into;
use crate::units::Samples;
use crate::module::*;
use crate::loader::Loader;

fn conv(x: f64) -> (u8, u8) {
    let val = (32767.0 * x.clamp(-1.0, 1.0)) as i16;
    ((val as u16 >> 8) as u8,
     (val as u16 & 0xffff) as u8)
}

pub struct Tape {
    f: BufWriter<File>,
    val: f64,
}

impl Tape {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 2 {
            return Err(format!("usage: {} fname", args[0]));
        }
        let fname = args[1];
        Ok( modref_new(Self::new(fname)) )
    }

    pub fn new(fname: &str) -> Self {
        let f = File::create(fname).expect("cant open"); // XXX more graceful error
        let buff = BufWriter::new(f);
        Tape {
            f: buff,
            val: 0.0,
        }
    }

    pub fn record(&mut self, m: &mut impl Module, outp: &str, time: impl Into<Samples>) -> Result<(), String> {
        let out_idx = m.output_idx("module", outp)?;
        let Samples(samples) = time.into();
        for _ in 1 .. samples {
            m.advance();
            let val = m.get_output(out_idx).ok_or("can't read gen output")?;
            let (a,b) = conv(val);
            self.f.write(&[a, b]).map_err(|_| "can't write file")?;
        }
        Ok(())
    }
}

impl Module for Tape {
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        (vec!["in".to_string()], 
         vec![])
    }

    fn get_output(&self, _idx: usize) -> Option<f64> {
        unreachable!();
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 {
            self.val = value;
        }
    }

    fn advance(&mut self) -> bool {
        let (a,b) = conv(self.val);
        self.f.write(&[a, b]).expect("cant write");
        return true;
    }
}

pub fn init(l: &mut Loader) {
    l.register("file", Tape::from_cmd);
}

