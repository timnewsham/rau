
use rau::gen::{Gen};
use rau::units::{Hz, Cent, Sec, Samples};
use rau::additive::{Gen as AddGen, Function};
use rau::simple::Gen as SimpGen;
use rau::ascii::{plot, plot1};
use rau::envelope::Envelope;
use rau::file::Tape;
use rau::filt::{Filter, FiltType};
use rau::module::{Rack, Module};
use rau::speaker::Speaker;
use rau::util::Mult;

#[allow(dead_code)]
fn visual_check_simple() {
    plot(&mut SimpGen::new(Function::SIN, Hz(2.0)));
    plot(&mut SimpGen::new(Function::TRI, Hz(2.0)));
    plot(&mut SimpGen::new(Function::SAWUP, Hz(2.0)));
    plot(&mut SimpGen::new(Function::SQUARE, Hz(2.0)));
}

#[allow(dead_code)]
fn visual_check_add() {
    plot(&mut AddGen::new(Function::SIN, Hz(2.0), 1));
    plot(&mut AddGen::new(Function::TRI, Hz(2.0), 10));
    plot(&mut AddGen::new(Function::SAWUP, Hz(2.0), 10));
    plot(&mut AddGen::new(Function::SQUARE, Hz(2.0), 10));

    // cost: 2
    let mut gen = AddGen::new(Function::SAWUP, Hz(10000.0), 40);
    debug_assert!(gen.cost() == 2);

    // verify phase continuity
    gen.set_freq(Hz(0.5));
    plot(&mut gen);
    gen.set_func(Function::SIN, 1);
    plot(&mut gen);
}

#[allow(dead_code)]
fn visual_check_env() {
    let mut env = Envelope::new(Samples(10), Samples(5), 0.3, Samples(10));
    for _ in 0..30 {
        env.set_input(0, 1.0); // gate
        env.advance();
        plot1(env.gen());
    }
    for _ in 0..20 {
        env.set_input(0, 0.0); // gate
        env.advance();
        plot1(env.gen());
    }
}

#[allow(dead_code)]
fn visual_check_filt() {
    let mut filt = Filter::new(FiltType::HP, Hz(5000.0), 1.0, 5.0);
    filt.set_input(0, 1.0);
    for _ in 0..20 {
        filt.advance();
        filt.set_input(0, 0.0);
        plot1(filt.get_output(0).unwrap());
    }
}

// sox -r 44100 -e signed -B -b 16 -c 1 out.s16 out.wav
#[allow(dead_code)]
fn make_file() {
    let mut gen = AddGen::new(Function::SAWUP, Hz(1.0), 40);
    //let mut gen = AddGen::new(Function::SIN, Hz(1.0), 1);

    let mut tape = Tape::new("out.s16");
    gen.set_freq(Hz(440.0));
    tape.record(&mut gen, Sec(0.25));
    gen.set_freq(Hz(880.0));
    tape.record(&mut gen, Sec(0.25));
}

// sox -r 44100 -e signed -B -b 16 -c 1 sweep.s16 sweep.wav
#[allow(dead_code)]
fn make_sweep() {
    //let mut gen = AddGen::new(Function::SQUARE, Hz(1.0), 40);
    let mut gen = AddGen::new(Function::SAWUP, Hz(1.0), 40);
    //let mut gen = AddGen::new(Function::SIN, Hz(1.0), 1);

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
    //let mut gen = SimpGen::new(Function::SQUARE, Hz(1.0));
    let mut gen = SimpGen::new(Function::SAWUP, Hz(1.0));
    //let mut gen = SimpGen::new(Function::SIN, Hz(1.0));

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
    let gen = rack.add_module(Box::new(AddGen::new(Function::SAWUP, Hz(1.0), 16)));
    let env = rack.add_module(Box::new(Envelope::new(Sec(0.1), Sec(0.2), 0.1, Sec(0.1))));
    let tape = rack.add_module(Box::new(Speaker::new()));
    let mul = rack.add_module(Box::new(Mult::new()));
    let filt = rack.add_module(Box::new(Filter::new(FiltType::LP, Hz(1000.0), 0.0, 0.1)));
    rack.add_wire(gen, "out", mul, "in1");
    rack.add_wire(env, "out", mul, "in2");
    rack.add_wire(mul, "out", filt, "in");
    rack.add_wire(filt, "out", tape, "in"); 

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
    let mut lfo_ = Box::new(SimpGen::new(Function::SAWUP, Hz(4.0)));
    lfo_.set_off(880.0);
    lfo_.set_amp(440.0);

    let mut rack = Rack::new();
    let osc = rack.add_module(Box::new(AddGen::new(Function::SIN, Hz(440.0), 1)));
    let lfo = rack.add_module(lfo_);
    //let tape = rack.add_module(Box::new(Tape::new("modtest.s16")));
    let speaker = rack.add_module(Box::new(Speaker::new()));

    rack.add_wire(lfo, "out", osc, "freq");
    //rack.add_wire(osc, "out", tape, "in");
    rack.add_wire(osc, "out", speaker, "in");
    for _ in 0..44100 { rack.advance(); }
}

fn main() {
    //visual_check_add();
    //visual_check_simple();
    visual_check_env();
    //visual_check_filt();

    //make_file();
    //make_sweep();
    //make_sweep2();
    //module_test();
    //make_tune();
}
