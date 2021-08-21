
use rau::units::{Hz, Cent, Sec, Samples};
use rau::additive::{Gen as AddGen, Function};
use rau::simple::Gen as SimpGen;
use rau::ascii::{plot, plot1};
use rau::envelope::Envelope;
use rau::file::Tape;
use rau::filt::{Filter, FiltType};
use rau::speaker::Speaker;
use rau::util::Mult;
use rau::loader;
use rau::module::{Rack, Module, modref_new};
use rau::wav::{Sample, read_wav};

#[allow(dead_code)]
fn visual_check_simple() -> Result<(), String> {
    plot(&mut SimpGen::new(Function::SIN, Hz(2.0)), "out")?;
    plot(&mut SimpGen::new(Function::TRI, Hz(2.0)), "out")?;
    plot(&mut SimpGen::new(Function::SAWUP, Hz(2.0)), "out")?;
    plot(&mut SimpGen::new(Function::SQUARE, Hz(2.0)), "out")?;
    Ok(())
}

#[allow(dead_code)]
fn visual_check_add() -> Result<(), String> {
    plot(&mut AddGen::new(Function::SIN, Hz(2.0), 1), "out")?;
    plot(&mut AddGen::new(Function::TRI, Hz(2.0), 10), "out")?;
    plot(&mut AddGen::new(Function::SAWUP, Hz(2.0), 10), "out")?;
    plot(&mut AddGen::new(Function::SQUARE, Hz(2.0), 10), "out")?;

    // cost: 2
    let mut gen = AddGen::new(Function::SAWUP, Hz(10000.0), 40);
    debug_assert!(gen.cost() == 2);

    // verify phase continuity
    gen.set_freq(Hz(0.5));
    plot(&mut gen, "out")?;
    gen.set_func(Function::SIN, 1);
    plot(&mut gen, "out")?;
    Ok(())
}

#[allow(dead_code)]
fn visual_check_env() -> Result<(), String> {
    let mut env = Envelope::new(Samples(10), Samples(5), 0.3, Samples(10));
    for _ in 0..30 {
        env.set_named_input("env", "gate", 1.0)?;
        env.advance();
        let val = env.get_named_output("env", "out")?;
        plot1(val);
    }
    for _ in 0..20 {
        env.set_named_input("env", "gate", 0.0)?;
        env.advance();
        let val = env.get_named_output("env", "out")?;
        plot1(val);
    }
    Ok(())
}

#[allow(dead_code)]
fn visual_check_filt() -> Result<(), String> {
    let mut filt = Filter::new(FiltType::HP, Hz(5000.0), 1.0, 5.0);
    filt.set_input(0, 1.0);
    for _ in 0..20 {
        filt.advance();
        filt.set_input(0, 0.0);
        plot1(filt.get_output(0).ok_or("cant get output")?);
    }
    Ok(())
}

// sox -r 44100 -e signed -B -b 16 -c 1 out.s16 out.wav
#[allow(dead_code)]
fn make_file() -> Result<(), String> {
    let mut gen = AddGen::new(Function::SAWUP, Hz(1.0), 40);
    //let mut gen = AddGen::new(Function::SIN, Hz(1.0), 1);

    let mut tape = Tape::new("out.s16");
    gen.set_freq(Hz(440.0));
    tape.record(&mut gen, "out", Sec(0.25))?;
    gen.set_freq(Hz(880.0));
    tape.record(&mut gen, "out", Sec(0.25))?;
    Ok(())
}

// sox -r 44100 -e signed -B -b 16 -c 1 sweep.s16 sweep.wav
#[allow(dead_code)]
fn make_sweep() -> Result<(), String> {
    //let mut gen = AddGen::new(Function::SQUARE, Hz(1.0), 40);
    let mut gen = AddGen::new(Function::SAWUP, Hz(1.0), 40);
    //let mut gen = AddGen::new(Function::SIN, Hz(1.0), 1);

    // 5 octaves up
    let mut tape = Tape::new("sweep.s16");
    for cent in 0..(1200 * 5) {
        gen.set_freq(Cent(cent as f64));
        // 0.5 seconds per octave
        tape.record(&mut gen, "out", Sec(0.5/1200.0))?;
    }
    // 5 octaves down
    for cent in 0..(1200 * 5) {
        gen.set_freq(Cent((1200*5 - cent) as f64));
        // 0.5 seconds per octave
        tape.record(&mut gen, "out", Sec(0.5/1200.0))?;
    }
    Ok(())
}

// sox -r 44100 -e signed -B -b 16 -c 1 sweep2.s16 sweep2.wav
#[allow(dead_code)]
fn make_sweep2() -> Result<(), String> {
    //let mut gen = SimpGen::new(Function::SQUARE, Hz(1.0));
    let mut gen = SimpGen::new(Function::SAWUP, Hz(1.0));
    //let mut gen = SimpGen::new(Function::SIN, Hz(1.0));

    // 5 octaves up
    let mut tape = Tape::new("sweep2.s16");
    for cent in 0..(1200 * 5) {
        //let Hz(freq) = Cent(cent as f64).into();
        //gen.set_named_input("gen", "freq", freq).unwrap();
        gen.set_freq(Cent(cent as f64));
        // 0.5 seconds per octave
        tape.record(&mut gen, "out", Sec(0.5/1200.0))?;
    }
    // 5 octaves down
    for cent in 0..(1200 * 5) {
        //let Hz(freq) = Cent((1200*5 - cent) as f64).into();
        //gen.set_named_input("gen", "freq", freq)?;
        gen.set_freq(Cent((1200*5 - cent) as f64));
        // 0.5 seconds per octave
        tape.record(&mut gen, "out", Sec(0.5/1200.0))?;
    }
    Ok(())
}

#[allow(dead_code)]
fn make_tune() -> Result<(), String> {
    let dur = Sec(0.25);
    //let mut tape = modref_new(Tape::new("tune.s16"));
    let mut rack = Rack::new();

    rack.add_module("gen", modref_new(AddGen::new(Function::SAWUP, Hz(1.0), 16)))?;
    rack.add_module("env", modref_new(Envelope::new(Sec(0.1), Sec(0.2), 0.1, Sec(0.1))))?;
    rack.add_module("speaker", modref_new(Speaker::new()))?;
    rack.add_module("mul", modref_new(Mult::new()))?;
    rack.add_module("filt", modref_new(Filter::new(FiltType::LP, Hz(1000.0), 0.0, 0.1)))?;

    rack.add_wire("gen", "out", "mul", "in1")?;
    rack.add_wire("env", "out", "mul", "in2")?;
    rack.add_wire("mul", "out", "filt", "in")?;
    rack.add_wire("filt", "out", "speaker", "left")?;
    rack.add_wire("filt", "out", "speaker", "right")?;

    let notes = vec![
        7,5,3,5,
        7,7,7,7,
        5,5,5,5,
        7,10,10,10];


    for note in notes {
        // XXX modules should have typed inputs with auto-conversions
        let Hz(freq) = Cent(note as f64 * 100.0).into();
        rack.set_input("gen", "freq", freq)?; // freq
        rack.set_input("env", "gate", 1.0)?; // gate
        rack.run(dur);
    }
    Ok(())
}

#[allow(dead_code)]
fn module_test() -> Result<(), String> {
    let mut lfo = SimpGen::new(Function::SAWUP, Hz(4.0));
    lfo.set_off(880.0);
    lfo.set_amp(440.0);

    let mut rack = Rack::new();

    rack.add_module("osc", modref_new(AddGen::new(Function::SIN, Hz(440.0), 1)))?;
    rack.add_module("lfo", modref_new(lfo))?;
    //rack.add_module("tape", modref_new(Tape::new("modtest.s16")))?;
    rack.add_module("speaker", modref_new(Speaker::new()))?;

    rack.add_wire("lfo", "out", "osc", "freq")?;
    //rack.add_wire("osc", "out", "tape", "in")?;
    rack.add_wire("osc", "out", "speaker", "left")?;
    rack.add_wire("osc", "out", "speaker", "right")?;
    for _ in 0..44100 { rack.advance(); }
    Ok(())
}

#[allow(dead_code)]
fn test_loader() -> Result<(), String> {
    let mut l = loader::Loader::new();
    let mut rack = l.load("test.txt")?;
    while rack.run(Samples(128)) {
        continue;
    }
    Ok(())
}

fn test_pitch() {
    // verify that the storage is shifting properly
    let mut p = rau::pitch::Pitch::new(Samples(10), Samples(3));
    for n in 0..9 {
        p.add_sample(n as f64);
    }
    println!("storage: {:?}", p.data);
    p.add_sample(9.0);
    println!("storage: {:?}", p.data);

    // verify that the window function is sane
    p.window.iter().copied().for_each(plot1);
}

fn show_pitch() {
    let mut p = rau::pitch::Pitch::new(Sec(0.050), Sec(0.010));
    let samps = read_wav("pitch.wav", 48000.0);
    let mut last_note = None;
    for Sample{left, right: _} in samps {
        let note = p.add_sample(left);
        if note != last_note {
            println!("{:?}", note);
        }
        last_note = note;
    }
}

fn main() -> Result<(), String> {
/*
    visual_check_add()?;
    visual_check_simple()?;
    visual_check_env()?;
    visual_check_filt()?;
*/

    //make_file()?;
    //make_sweep()?;
    //make_sweep2()?;
    //module_test()?;

    //make_tune()?;
    //test_loader()?;
    
    //test_pitch();
    show_pitch();

    Ok(())
}
