
use rau::units::{Hz, Sec, Samples};
use rau::additive::{Gen as AddGen, Function};
use rau::envelope::Envelope;
use rau::filt::{Filter, FiltType};
use rau::keyboard::{Keyboard};
use rau::speaker::Speaker;
use rau::util::Mult;
use rau::module::{Rack, modref_new};

// build a simple synth and run it until someine hits "Esc".
fn run() -> Result<(), String> {
    let mut rack = Rack::new();

    rack.add_module("key", modref_new(Keyboard::new(Sec(0.01))))?;
    rack.add_module("gen", modref_new(AddGen::new(Function::SAWUP, Hz(1.0), 16)))?;
    rack.add_module("env", modref_new(Envelope::new(Sec(0.05), Sec(0.2), 0.4, Sec(0.5))))?;
    rack.add_module("speaker", modref_new(Speaker::new()))?;
    rack.add_module("mul", modref_new(Mult::new()))?;
    rack.add_module("filt", modref_new(Filter::new(FiltType::LP, Hz(1000.0), 0.0, 0.1)))?;

    rack.add_wire("key", "out", "gen", "freq")?;
    rack.add_wire("key", "gate", "env", "gate")?;
    rack.add_wire("gen", "out", "mul", "in1")?;
    rack.add_wire("env", "out", "mul", "in2")?;
    rack.add_wire("mul", "out", "filt", "in")?;
    rack.add_wire("filt", "out", "speaker", "left")?;
    rack.add_wire("filt", "out", "speaker", "right")?;

    println!("running");
    while rack.run(Samples(128)) {
        continue;
    }
    println!("done");
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("error {}", e);
    }
}

