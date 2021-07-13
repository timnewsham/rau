
use crate::freq::SAMPLE_RATE;
use crate::gen::Gen;

fn repeat(ch: char, n: i64) {
    for _ in 0..n {
        print!("{}", ch);
    }
}

fn plot1(x: f64) {
    let center = 40;
    let width = 74;
    let mut off = (((x * width as f64) / 2.0) as i64) + center;
    if off < 1 { off = 1; }
    if off > 78 { off = 78; }
    if off == center {
        repeat(' ', center-1);
        repeat('*', 1);
        repeat('\n', 1);
    } else if off < center {
        repeat(' ', off-1);
        repeat('*', 1);
        repeat('-', center-off-1);
        repeat('|', 1);
        repeat('\n', 1);
    } else {
        repeat(' ', center-1);
        repeat('|', 1);
        repeat('-', off-center-1);
        repeat('*', 1);
        repeat('\n', 1);
    }
}

pub fn plot(gen: &mut impl Gen) {
    let DECIMATE = 44100 / 30;
    for n in 0 .. SAMPLE_RATE as i64 {
        if n % DECIMATE == 0 {
            plot1(gen.gen());
        }
        gen.advance();
    }
    println!("Cost {:?}", gen.cost());
    println!("");
}

