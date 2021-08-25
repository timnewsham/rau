
use std::cmp::Ordering;
use crate::units::SAMPLE_RATE;
use crate::module::*;

fn repeat(s: &mut String, ch: char, n: i64) {
    for _ in 0..n {
        s.push(ch);
    }
}

pub fn format1(x: f64) -> String {
    let center = 40;
    let width = 74;
    let mut off = (((x * width as f64) / 2.0) as i64) + center;
    if off < 1 { off = 1; }
    if off > 78 { off = 78; }
    let mut s = String::new();
    match off.cmp(&center) {
        Ordering::Equal => {
            repeat(&mut s, ' ', center-1);
            repeat(&mut s, '*', 1);
        },
        Ordering::Less => {
            repeat(&mut s, ' ', off-1);
            repeat(&mut s, '*', 1);
            repeat(&mut s, '-', center-off-1);
            repeat(&mut s, '|', 1);
        },
        Ordering::Greater => {
            repeat(&mut s, ' ', center-1);
            repeat(&mut s, '|', 1);
            repeat(&mut s, '-', off-center-1);
            repeat(&mut s, '*', 1);
        },
    }
    s
}

pub fn plot1(x: f64) {
    println!("{}", format1(x));
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

