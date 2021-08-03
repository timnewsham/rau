
mod additive;
mod envelope;
mod simple;
mod ascii;
mod file;
mod gen;
mod module;
mod speaker;
mod units;
mod util;

use crate::gen::Gen;
use crate::units::{Hz, Cent, Sec, Samples};
use crate::additive::Gen as AddGen;
use crate::simple::Gen as SimpGen;
use crate::ascii::plot;
use crate::envelope::Envelope;
use crate::file::Tape;
use crate::module::{Rack, Module};
use crate::speaker::Speaker;
use crate::util::Mult;

#[allow(dead_code)]
fn visual_check_simple() {
    plot(&mut SimpGen::new_sine(Hz(2.0)));
    plot(&mut SimpGen::new_triangle(Hz(2.0)));
    plot(&mut SimpGen::new_saw_up(Hz(2.0)));
    plot(&mut SimpGen::new_square(Hz(2.0)));
}

#[allow(dead_code)]
fn visual_check_add() {
    plot(&mut AddGen::new_sine(Hz(2.0)));
    plot(&mut AddGen::new_triangle(Hz(2.0), 10));
    plot(&mut AddGen::new_saw_up(Hz(2.0), 10));
    plot(&mut AddGen::new_square(Hz(2.0), 10));

    // cost: 2
    let mut gen = AddGen::new_saw_up(Hz(10000.0), 40);
    debug_assert!(gen.cost() == 2);

    // verify phase continuity
    gen.set_freq(Hz(0.5));
    ascii::plot(&mut gen);
    gen.set_sine();
    ascii::plot(&mut gen);
}

#[allow(dead_code)]
fn visual_check_env() {
    let mut env = Envelope::new(Samples(10), Samples(5), 0.3, Samples(10));
    env.set_gate(true);
    for _ in 0..30 {
        env.advance();
        ascii::plot1(env.gen());
    }
    env.set_gate(false);
    for _ in 0..20 {
        env.advance();
        ascii::plot1(env.gen());
    }
}

// sox -r 44100 -e signed -B -b 16 -c 1 out.s16 out.wav
#[allow(dead_code)]
fn make_file() {
    let mut gen = AddGen::new_saw_up(Hz(1.0), 40);
    //let mut gen = AddGen::new_sine(Hz(1.0));

    let mut tape = Tape::new("out.s16");
    gen.set_freq(Hz(440.0));
    tape.record(&mut gen, Sec(0.25));
    gen.set_freq(Hz(880.0));
    tape.record(&mut gen, Sec(0.25));
}

// sox -r 44100 -e signed -B -b 16 -c 1 sweep.s16 sweep.wav
#[allow(dead_code)]
fn make_sweep() {
    //let mut gen = AddGen::new_square(Hz(1.0), 40);
    let mut gen = AddGen::new_saw_up(Hz(1.0), 40);
    //let mut gen = AddGen::new_sine(Hz(1.0));

    // 5 octaves up
    let mut tape = Tape::new("sweep.s16");
    for cent in 0..(1200 * 5) {
        gen.set_freq(Cent(cent as f64));
        // 0.5 seconds per octave
        tape.record(&mut gen, Sec(0.5/1200.0));
    }
    // 5 octaves down
    for cent in 0..(1200 * 5) {
        gen.set_freq(Cent((1200*5 - cent) as f64));
        // 0.5 seconds per octave
        tape.record(&mut gen, Sec(0.5/1200.0));
    }
}

// sox -r 44100 -e signed -B -b 16 -c 1 sweep2.s16 sweep2.wav
#[allow(dead_code)]
fn make_sweep2() {
    //let mut gen = SimpGen::new_square(Hz(1.0));
    let mut gen = SimpGen::new_saw_up(Hz(1.0));
    //let mut gen = SimpGen::new_sine(Hz(1.0));

    // 5 octaves up
    let mut tape = Tape::new("sweep2.s16");
    for cent in 0..(1200 * 5) {
        gen.set_freq(Cent(cent as f64));
        // 0.5 seconds per octave
        tape.record(&mut gen, Sec(0.5/1200.0));
    }
    // 5 octaves down
    for cent in 0..(1200 * 5) {
        gen.set_freq(Cent((1200*5 - cent) as f64));
        // 0.5 seconds per octave
        tape.record(&mut gen, Sec(0.5/1200.0));
    }
}

#[allow(dead_code)]
fn make_tune() {
    let dur = Sec(0.25);
    //let mut tape = Box::new(Tape::new("tune.s16"));
    let mut rack = Rack::new();
    let gen = rack.add_module(Box::new(AddGen::new_sine(Hz(1.0))));
    let env = rack.add_module(Box::new(Envelope::new(Sec(0.1), Sec(0.2), 0.1, Sec(0.1))));
    let tape = rack.add_module(Box::new(Speaker::new()));
    let mul = rack.add_module(Box::new(Mult::new()));
    rack.add_wire(gen, "out", mul, "in1");
    rack.add_wire(env, "out", mul, "in2");
    rack.add_wire(mul, "out", tape, "in");
    //rack.add_wire(gen, "out", tape, "in");

    let notes = vec![
        7,5,3,5,
        7,7,7,7,
        5,5,5,5,
        7,10,10,10];


    for note in notes {
        // XXX modules should have typed inputs with auto-conversions
        let Hz(freq) = Cent(note as f64 * 100.0).into();
        rack.set_input(gen, 0, freq); // freq
        rack.set_input(env, 0, 1.0); // gate
        rack.run(dur);
    }
}

#[allow(dead_code)]
fn module_test() {
    let mut lfo_ = Box::new(SimpGen::new_saw_up(Hz(4.0)));
    lfo_.set_off(880.0);
    lfo_.set_amp(440.0);

    let mut rack = Rack::new();
    let osc = rack.add_module(Box::new(AddGen::new_sine(Hz(440.0))));
    let lfo = rack.add_module(lfo_);
    //let tape = rack.add_module(Box::new(Tape::new("modtest.s16")));
    let speaker = rack.add_module(Box::new(Speaker::new()));

    rack.add_wire(lfo, "out", osc, "freq");
    //rack.add_wire(osc, "out", tape, "in");
    rack.add_wire(osc, "out", speaker, "in");
    for _ in 0..44100 { rack.advance(); }
}

fn main() {
    //make_file();
    //make_sweep();
    //make_sweep2();
    make_tune();

    //module_test();

    //visual_check_add();
    //visual_check_simple();
    //visual_check_env();
}
