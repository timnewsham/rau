
use std::cmp::Ordering;
use crate::units::SAMPLE_RATE;
use crate::module::*;

fn repeat(ch: char, n: i64) {
    for _ in 0..n {
        print!("{}", ch);
    }
}

pub fn plot1(x: f64) {
    let center = 40;
    let width = 74;
    let mut off = (((x * width as f64) / 2.0) as i64) + center;
    if off < 1 { off = 1; }
    if off > 78 { off = 78; }
    match off.cmp(&center) {
        Ordering::Equal => {
            repeat(' ', center-1);
            repeat('*', 1);
        },
        Ordering::Less => {
            repeat(' ', off-1);
            repeat('*', 1);
            repeat('-', center-off-1);
            repeat('|', 1);
        },
        Ordering::Greater => {
            repeat(' ', center-1);
            repeat('|', 1);
            repeat('-', off-center-1);
            repeat('*', 1);
        },
    }
    repeat('\n', 1);
}

const DECIMATE : i64 = 44100 / 30;

pub fn plot(m: &mut impl Module, outp: &str) -> Result<(), String> {
    let out_idx = m.output_idx("module", outp)?;
    for n in 0 .. SAMPLE_RATE as i64 {
        if n % DECIMATE == 0 {
            let val = m.get_output(out_idx).ok_or("Can't read module output")?;
            plot1(val);
        }
        m.advance();
    }
    //println!("Cost {:?}", gen.cost());
    println!();
    Ok(())
}

