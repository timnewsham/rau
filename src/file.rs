
use std::fs;
use std::convert::Into;
use crate::units::Samples;
use crate::gen::Gen;

pub fn clamp(x: f64) -> f64 {
    if x < -1.0 {
        -1.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}

// convert float samples to s16 samples as big-endian
fn convert(data: &Vec<f64>, to: &mut Vec<u8>) {
    for samp in data.iter() {
        let val = (32767.0 * clamp(*samp)) as i16;
        to.push((val as u16 >> 8) as u8);
        to.push((val as u16 & 0xffff) as u8);
    }
}

pub struct Tape {
    fname: String,
    samples: Vec<f64>,
}

impl Tape {
    pub fn new(fname: &str) -> Self {
        Tape {
            fname: fname.to_owned(),
            samples: Vec::new(),
        }
    }

    pub fn record(&mut self, gen: &mut impl Gen, time: impl Into<Samples>) {
        let samples : Samples = time.into();                                        
        for _ in 1 .. samples.0 {                                                   
            self.samples.push(gen.gen());                                                   
            gen.advance();                                                          
        }
    }                                                                           
}

impl Drop for Tape {
    // write file when we drop
    fn drop(&mut self) {
        let mut out = Vec::new();
        convert(&self.samples, &mut out);
        fs::write(&self.fname, out).expect("couldnt write file");
    }
}

