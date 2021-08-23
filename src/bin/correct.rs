
use std::env;
use rau::pitch::PitchCorrect;
use rau::speaker::{Speaker, SamplePlayer};
use rau::wav::{read_wav_at, Sample};
use rau::units::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut fname = "pitch.wav";
    if args.len() > 1 {
        fname = &args[1];
    }

    println!("correcting {}", fname);
    let samples = read_wav_at(fname, SAMPLE_RATE);
    let mut c = PitchCorrect::new(Cent(-(2400.0 + 500.0)), Cent(1200.0), 0.75);
    let mut speaker = Speaker::new();
    for Sample{left, right: _} in samples {
        if let Some(outs) = c.process(left) {
            for out in outs {
                speaker.play(Sample{ left: out, right: out });
            }
        }
    }
}

