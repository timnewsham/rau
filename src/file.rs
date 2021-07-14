
use std::fs::File;
use std::io::{Write,BufWriter};
use std::convert::Into;
use crate::units::Samples;
use crate::gen::Gen;
use crate::module;

fn conv(x: f64) -> (u8, u8) {
    let val = (32767.0 * x.clamp(-1.0, 1.0)) as i16;
    ((val as u16 >> 8) as u8,
     (val as u16 & 0xffff) as u8)
}

pub struct Tape {
    f: BufWriter<File>,
}

impl Tape {
    pub fn new(fname: &str) -> Self {
        let f = File::create(fname).expect("cant open");
        let buff = BufWriter::new(f);
        Tape {
            f: buff,
        }
    }

    pub fn record(&mut self, gen: &mut impl Gen, time: impl Into<Samples>) {
        let samples : Samples = time.into();
        for _ in 1 .. samples.0 {
            let (a,b) = conv(gen.gen());
            self.f.write(&[a, b]).expect("cant write");
            gen.advance();
        }
    }
}

impl module::Module for Tape {
    fn get_terminals(&self) -> (Vec<module::TerminalDescr>, Vec<module::TerminalDescr>) {
        (vec!["in".to_string()], 
         vec![])
    }

    fn get_output(&self, _idx: usize) -> Option<f64> {
        unreachable!();
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 {
            let (a,b) = conv(value);
            self.f.write(&[a, b]).expect("cant write");
        }
    }

    fn advance(&mut self) {
    }
}
