
use std::fs;

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
pub fn convert(data: &Vec<f64>, to: &mut Vec<u8>) {
    for samp in data.iter() {
        let val = (32767.0 * clamp(*samp)) as i16;
        to.push((val as u16 >> 8) as u8);
        to.push((val as u16 & 0xffff) as u8);
    }
}

pub fn writeFile(fname: &str, data: &Vec<f64>) {
    let mut out = Vec::new();
    convert(data, &mut out);
    fs::write(fname, out);
}
