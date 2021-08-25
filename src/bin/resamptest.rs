
use std::f64::consts::PI;
use rau::units::*;
use rau::resampler::*;
use rau::ascii::format1;

// test the resampler to see how it affects phase.
fn main() {
    let order = 16;

    let freq = SAMPLE_RATE / 40.0;
    for (up,down) in [(16,5), (101,100), (2,1), (7,4), (4,7), (1,3), (1,2)] {
        println!("resample {}:{}", up, down);
        let mut r = Resampler::new(up, down, 50.0, 0.7, order);

        let mut xs: Vec<f64> = Vec::new();
        let mut ys: Vec<f64> = Vec::new();

        for n in 0..300 {
            let x = 0.5 * (2.0 * PI * (n as f64) * freq / SAMPLE_RATE).sin();
            xs.push(x);
            r.resample(x, |y| ys.push(y));
        }

        let cnt = 5;
        for n in 0..cnt {
            println!("{:70} {}", format1(xs[n]), format1(ys[n]));
        }
    }
}

