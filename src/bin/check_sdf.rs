
use rau::corr::*;

pub fn main() {
    let mut sdf = NaiveSDF::new(32, 32);
    let mut fsdf = SDF::new(32, 32);

    let buf: Vec<f64> = (0..32).map(|n| (n as f64 * 6.28 / 10.0).sin()).collect();

    sdf.process(&buf);
    fsdf.process(&buf);

    let mut err = 0.0;
    for n in 0..32 {
        let e2 = (sdf.buf[n] - fsdf.buf[n]).powi(2);
        err += e2;
        println!("sdf {}\tfsdf {}\terr2 {}", sdf.buf[n], fsdf.buf[n], e2);
    }
    println!("error {}", err);
}
