
use std::f64::consts::PI;
use rau::units::*;
use rau::pitch::*;
use rau::ascii::format1;

// increase freq by a few times so that we have to loop input several times
fn pitch_up(_: Option<Cent>) -> f64 { 3.2 }

/*
comments:
I dont see any discontinuities.. which is great.
But I do notice a few problems
   - the resampler takes a while to warm up. we should warm up the resampler before using it!
   - I see non-uniform amplitude during the cross-fading.  not sure yet if this is due to
     a phase issue when mixixng the overlap, or if it is because of resampler warmup.
     For simple sine waves repitched up by a constant factor, this should not happen!
*/

// test the re-pitching (pitch correction) module
// looking for phase misalignments while looping the input and while
// cross fading the windows.
fn main() {
    let freq = SAMPLE_RATE / 40.0;
    let mut c = PitchCorrect::new(pitch_up, Hz(freq * 0.9), Hz(freq * 1.5), 0.3);
    let mut cnt = 0;

    let mut overlap: Vec<f64> = c.overlap.iter().copied().collect();

    for n in 0.. {
        let x = 0.5 * (2.0 * PI * (n as f64) * freq / SAMPLE_RATE).sin();
        //rau::ascii::plot1(x);
        if let Some(outs) = c.process(x) {
            let midtrans = overlap.len() / 2;
            for (n,out) in outs.iter().enumerate() {
                if n == midtrans {
                    println!("mid");
                }
                if n < overlap.len() {
                    println!("{:70} {}", format1(*out), format1(overlap[n]));
                    //println!("{:70} {}", format1(*out), format1(c.p.data[n]));
                } else {
                    println!("{:70}", format1(*out));
                }
            }

            println!("n {}, period {:?}, data size {}", n, c.inputperiod, outs.len());
            //println!("warm {:?}", c.warmup);
            overlap.copy_from_slice(&c.overlap);
            cnt += 1;
            if cnt == 4 { break; }
        }
    }
}

