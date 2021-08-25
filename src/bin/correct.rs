
use std::env;
use rau::speaker::{Speaker, SamplePlayer};
use rau::wav::{read_wav_at, Sample};
use rau::module::Module;
use rau::file::Tape;
use rau::units::*;
use rau::pitch::*;

#[allow(dead_code)]
fn pitch_up(_: Option<Cent>) -> f64 { 3.0 / 2.0 }

#[allow(dead_code)]
fn pitch_down(_: Option<Cent>) -> f64 { 2.0 / 3.0 }

#[allow(dead_code)]
fn nop(_: Option<Cent>) -> f64 { 1.0 }

// quantize to nearest "A"
#[allow(dead_code)]
fn mono_a(note: Option<Cent>) -> f64 {
    match note {
        None => 1.0,
        Some(note) => {
            let octaves = note.0 / 1200.0;
            let corrected = octaves.round();
            let note2 = Cent(1200.0 * corrected);
            freq_ratio(note, note2)
        },
    }
}

fn pickfn(word: &str) -> CorrectFn {
    match word {
        "nop" => nop,
        "quantize" => quantize_note,
        "mono_a" => mono_a,
        "up" => pitch_up,
        "down" => pitch_down,
        _ => panic!("unknown pitch function"),
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let defmode = "quantize";
    let deffn = "pitch_wav".to_string();
    let (mode, fname) = match args.len() {
        0|1 => (pickfn(defmode), &deffn),
        2 =>   (pickfn(defmode), &args[1]),
        3 =>   (pickfn(&args[1]), &args[2]),
        _ =>   panic!("usage: prog [func] [file]"),
    };
        
    println!("correcting {}", fname);
    let samples = read_wav_at(fname, SAMPLE_RATE);
    //let mut c = PitchCorrect::new(quantize_note, Cent(-(2400.0 + 500.0)), Cent(1200.0), 0.75);
    //let mut c = PitchCorrect::new(quantize_note, Cent(-(2400.0 + 500.0)), Cent(1200.0), 0.75);
    //let mut c = PitchCorrect::new(pitch_up, Cent(-(2400.0 + 500.0)), Cent(1200.0), 0.75);
    //let mut c = PitchCorrect::new(nop, Cent(-(2400.0 + 500.0)), Cent(1200.0), 0.75);
    //let mut c = PitchCorrect::new(mono_a, Cent(-(2400.0 + 500.0)), Cent(1200.0), 0.75);
    let mut c = PitchCorrect::new(mode, Cent(-(2400.0 + 500.0)), Cent(1200.0), 0.75);
    let mut speaker = Speaker::new();
    let mut tape = Tape::new("repitched.s16");
    for Sample{left, right: _} in samples {
        if let Some(outs) = c.process(left) {
            for out in outs {
                speaker.play(Sample{ left: out, right: out });

                tape.set_input(0, out);
                tape.advance();
            }
        }
    }
}

