
use std::fs::File;
use std::io::Write;
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

/*
// convert float samples to s16 samples as big-endian
fn convert(data: &Vec<f64>, to: &mut Vec<u8>) {
    for samp in data.iter() {
        let val = (32767.0 * clamp(*samp)) as i16;
        to.push((val as u16 >> 8) as u8);
        to.push((val as u16 & 0xffff) as u8);
    }
}
*/

fn conv(x: f64) -> (u8, u8) {
    let val = (32767.0 * clamp(x)) as i16;
    ((val as u16 >> 8) as u8,
     (val as u16 & 0xffff) as u8)
}

pub struct Tape {
    f: File,
}

impl Tape {
    pub fn new(fname: &str) -> Self {
        let f = File::create(fname).expect("cant open");
        Tape {
            f: f,
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

