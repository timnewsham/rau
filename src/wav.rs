
use std::fs::File;
use wav::{self, bit_depth::BitDepth};
use crate::speaker::Sample;

fn cvt_pairs<T: Copy, F: Fn(T) -> f64>(vs: &Vec<T>, cvt: F) -> Vec<Sample> {
    (0..vs.len())
        .step_by(2)
        .map(|i| Sample{ right: cvt(vs[i]), left: cvt(vs[i+1]) })
        .collect()
}

fn convert_samples(wavsamps: &BitDepth) -> Vec<Sample> {
    match wavsamps {
        BitDepth::Eight(vs) => cvt_pairs(vs, |vu8| (vu8 as f64 - 128.0) / 128.0),
        BitDepth::Sixteen(vs) => cvt_pairs(vs, |vi16| vi16 as f64 / 32768.0),
        BitDepth::TwentyFour(vs) => cvt_pairs(vs, |vi32| vi32 as f64 / 16777216.0),
        BitDepth::ThirtyTwoFloat(vs) => cvt_pairs(vs, |vf32| vf32 as f64),
        BitDepth::Empty => panic!("can't process empty samples"),
    }
}

pub fn read_wav(path: &str, rate: f64) -> Vec<Sample> {
    let mut inp = File::open(path).expect("couldnt open file");
    let (hdr, dat) = wav::read(&mut inp).expect("couldn't read samples");
    assert!(hdr.channel_count == 2 && rate as u32 == hdr.sampling_rate);
    convert_samples(&dat)
}
