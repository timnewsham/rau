
mod ascii;
mod file;
mod gen;
mod units;

use crate::units::{Hz, Cent, Sec};
use crate::gen::HarmonicGenerator;
use crate::ascii::plot;
use crate::file::Tape;

fn visual_check() {
    plot(&mut HarmonicGenerator::new_sine(Hz(2.0)));
    plot(&mut HarmonicGenerator::new_triangle(Hz(2.0), 10));
    plot(&mut HarmonicGenerator::new_saw_up(Hz(2.0), 10));
    plot(&mut HarmonicGenerator::new_square(Hz(2.0), 10));

    // cost: 2
    let mut gen = HarmonicGenerator::new_saw_up(Hz(10000.0), 40);
    debug_assert!(gen.cost() == 2);

    // verify phase continuity
    gen.set_freq(Hz(0.5));
    ascii::plot(&mut gen);
    gen.set_sine();
    ascii::plot(&mut gen);
}

// sox -r 44100 -e signed -B -b 16 -c 1 out.s16 out.wav
fn make_file() {
    let mut gen = HarmonicGenerator::new_saw_up(Hz(1.0), 40);
    //let mut gen = HarmonicGenerator::new_sine(Hz(1.0));

    let mut tape = Tape::new("out.s16");
    gen.set_freq(Hz(440.0));
    tape.record(&mut gen, Sec(0.25));
    gen.set_freq(Hz(880.0));
    tape.record(&mut gen, Sec(0.25));
}

// sox -r 44100 -e signed -B -b 16 -c 1 sweep.s16 sweep.wav
fn make_sweep() {
    //let mut gen = HarmonicGenerator::new_square(Hz(1.0), 40);
    let mut gen = HarmonicGenerator::new_saw_up(Hz(1.0), 40);
    //let mut gen = HarmonicGenerator::new_sine(Hz(1.0));

    // 5 octaves
    let mut tape = Tape::new("sweep.s16");
    for cent in 0..(1200 * 5) {
        gen.set_freq(Cent(cent as f64));
        // an octave a second
        tape.record(&mut gen, Sec(1.0/1200.0));
    }
}

fn make_tune() {
    let dur = Sec(0.25);
    let mut gen = HarmonicGenerator::new_sine(Hz(1.0));

    let notes = vec![
        7,5,3,5,
        7,7,7,7,
        5,5,5,5,
        7,10,10,10];

    let mut tape = Tape::new("tune.s16");
    for note in notes {
        gen.set_freq(Cent(note as f64 * 100.0));
        tape.record(&mut gen, dur);
    }
}

fn main() {
    make_file();
    make_sweep();
    make_tune();
    visual_check();
}
