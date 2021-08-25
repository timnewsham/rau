
use std::f64::consts::PI;
use rau::units::*;
use rau::pitch::*;
use rau::ascii::format1;

// increase freq by a few times so that we have to loop input several times
#[allow(dead_code)]
fn pitch_up(_: Option<Cent>) -> f64 { 3.2 }
#[allow(dead_code)]
fn pitch_down(_: Option<Cent>) -> f64 { 1.0 / 3.2 }

/*
 Test the pitch correction code to make sure the phases line up when looping
 and when overlapping windows of output.
*/


#[allow(dead_code)]
fn win_tests_overlaps(n: usize, c: &PitchCorrect, outs: &Vec<f64>, overlap: &Vec<f64>) {
    let midtrans = overlap.len() / 2;
    println!("mid {}", midtrans);
    for (n,out) in outs.iter().enumerate() {
        if n == midtrans {
            println!("mid");
        }
        if n < overlap.len() {
            println!("{:70} {}", format1(*out), format1(overlap[n]));
        } else {
            println!("{:70}", format1(*out));
        }
    }

    println!("n {}, input period {:?}, data size {}", n, c.inputperiod, outs.len());
}

#[allow(dead_code)]
fn win_tests_inputs(n: usize, c: &PitchCorrect, outs: &Vec<f64>, overlap: &Vec<f64>) {
    let midtrans = overlap.len() / 2;
    println!("mid {}", midtrans);
    for (n,out) in outs.iter().enumerate() {
        if n == midtrans {
            println!("mid");
        }
        println!("{:70} {}", format1(*out), format1(c.p.data[n]));
    }

    println!("n {}, input period {:?}, data size {}", n, c.inputperiod, outs.len());
}

// to use, force delay to zero, use zero pad instead of overlap pad, and dont mix overlap in.
#[allow(dead_code)]
fn win_tests_phases(_: usize, c: &PitchCorrect, outs: &Vec<f64>, overlap: &Vec<f64>) {
    println!("{:70} {}", "input", "output");
    for j in 0..5 {
        println!("{:70} {}", format1(c.p.data[j]), format1(outs[j]));
    }
    println!("");

    println!("{:70} {}    overlapsz {}, mid {}", "midoutput", "midoverlap", overlap.len(), overlap.len()/2);
    let midtrans = overlap.len() / 2;
    for j in 0..5 {
        println!("{:70} {}", format1(outs[j+midtrans]), format1(overlap[j+midtrans]));
    }

}

// test the re-pitching (pitch correction) module
// looking for phase misalignments while looping the input and while
// cross fading the windows.
fn main() {
    let test = 1;
    let freq;
    let mut c = match test {
        1 => {
            freq = SAMPLE_RATE / 40.0;
            PitchCorrect::new(pitch_up, Hz(freq * 0.9), Hz(freq * 1.5), 0.35)
        },
        2 => {
            freq = SAMPLE_RATE / 5.0;
            PitchCorrect::new(pitch_down, Hz(freq * 0.3), Hz(freq * 1.5), 0.35)
        },
        _ => panic!("bad choice"),
    };

    let mut cnt = 0;
    let mut overlap: Vec<f64> = c.overlap.iter().copied().collect();

    println!("true input period {}", SAMPLE_RATE / freq);
    for n in 0.. {
        let x = 0.5 * (2.0 * PI * (n as f64) * freq / SAMPLE_RATE).sin();
        //rau::ascii::plot1(x);
        if let Some(outs) = c.process(x) {
            //win_tests_inputs(n, &c, &outs, &overlap);
            win_tests_overlaps(n, &c, &outs, &overlap);
            //win_tests_phases(n, &c, &outs, &overlap);

            overlap.copy_from_slice(&c.overlap);
            cnt += 1;
            if cnt == 7 { break; }
        }
    }
}

