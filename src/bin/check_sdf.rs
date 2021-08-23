
use rau::corr::*;

pub fn main() {
    let mut ref_sdf = NaiveSDF::new(32, 32);
    let mut sdf = SDF::new(32, 32);

    let buf: Vec<f64> = (0..32).map(|n| (n as f64 * 6.28 / 10.0).sin()).collect();

    ref_sdf.process(&buf);
    sdf.process(&buf, false);

    let mut err = 0.0;
    for n in 0..32 {
        let e2 = (ref_sdf.buf[n] - sdf.buf[n]).powi(2);
        err += e2;
        println!("ref sdf {}\tsdf {}\terr2 {}", ref_sdf.buf[n], sdf.buf[n], e2);
    }
    println!("error {}", err);

    println!("");
    sdf.process(&buf, true);
    for n in 0..32 {
        println!("norm sdf {}", sdf.buf[n]);
    }
}
