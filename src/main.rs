
mod ascii;
mod file;
mod freq;
mod gen;

fn record(gen: &mut impl gen::Gen, seconds: f64, tape: &mut Vec<f64>) {
    let samples = (seconds * freq::SAMPLE_RATE) as usize;
    for _ in 1..samples {
        tape.push(gen.gen());
        gen.advance();
    }
}

fn visual_check() {
    ascii::plot(&mut gen::HarmonicGenerator::new_sine(2.0));
    ascii::plot(&mut gen::HarmonicGenerator::new_triangle(2.0, 10));
    ascii::plot(&mut gen::HarmonicGenerator::new_saw_up(2.0, 10));
    ascii::plot(&mut gen::HarmonicGenerator::new_square(2.0, 10));

    // cost: 2
    let mut gen = gen::HarmonicGenerator::new_saw_up(10000.0, 40);
    debug_assert!(gen.cost() == 2);

    // verify phase continuity
    gen.set_freq(0.5);
    ascii::plot(&mut gen);
    gen.set_sine();
    ascii::plot(&mut gen);
}

// sox -r 44100 -e signed -B -b 16 -c 1 out.s16 out.wav
fn make_file() {
    let mut tape = Vec::new();
    let mut gen = gen::HarmonicGenerator::new_saw_up(10000.0, 40);
    //let mut gen = gen::HarmonicGenerator::new_sine(1.0);

    gen.set_freq(440.0);
    record(&mut gen, 0.25, &mut tape);
    gen.set_freq(880.0);
    record(&mut gen, 0.25, &mut tape);
    file::writeFile("out.s16", &tape);
}

// sox -r 44100 -e signed -B -b 16 -c 1 sweep.s16 sweep.wav
fn make_sweep() {
    let mut tape = Vec::new();
    //let mut gen = gen::HarmonicGenerator::new_square(10000.0, 40);
    let mut gen = gen::HarmonicGenerator::new_saw_up(10000.0, 40);
    //let mut gen = gen::HarmonicGenerator::new_sine(1.0);

    // 5 octaves
    for cent in 0..(1200 * 5) {
        gen.set_note(cent as f64);
        // an octave a second
        record(&mut gen, 1.0/1200.0, &mut tape);
    }
    file::writeFile("sweep.s16", &tape);
}

fn make_tune() {
    let dur = 0.25;
    let mut tape = Vec::new();
    let mut gen = gen::HarmonicGenerator::new_sine(1.0);

    let notes = vec![
        7,5,3,5,
        7,7,7,7,
        5,5,5,5,
        7,10,10,10];
    for note in notes {
        gen.set_note(note as f64 * 100.0);
        record(&mut gen, dur, &mut tape);
    }
    file::writeFile("tune.s16", &tape);
}

fn main() {
    make_file();
    make_sweep();
    make_tune();
    visual_check();
}
