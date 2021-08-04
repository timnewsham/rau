
use rau::units::{Hz, Sec, Samples};
use rau::additive::{Gen as AddGen, Function};
use rau::envelope::Envelope;
use rau::filt::{Filter, FiltType};
use rau::keyboard::{Keyboard};
use rau::module::Rack;
use rau::speaker::Speaker;
use rau::util::Mult;

// build a simple synth and run it until someine hits "Esc".
fn main() {
    let mut rack = Rack::new();
    let key = rack.add_module(Box::new(Keyboard::new(Sec(0.01))));
    let gen = rack.add_module(Box::new(AddGen::new(Function::SAWUP, Hz(1.0), 16)));
    let env = rack.add_module(Box::new(Envelope::new(Sec(0.05), Sec(0.2), 0.4, Sec(0.5))));
    let tape = rack.add_module(Box::new(Speaker::new()));
    let mul = rack.add_module(Box::new(Mult::new()));
    let filt = rack.add_module(Box::new(Filter::new(FiltType::LP, Hz(1000.0), 0.0, 0.1)));
    rack.add_wire(key, "out", gen, "freq");
    rack.add_wire(key, "gate", env, "gate");
    rack.add_wire(gen, "out", mul, "in1");
    rack.add_wire(env, "out", mul, "in2");
    rack.add_wire(mul, "out", filt, "in");
    rack.add_wire(filt, "out", tape, "in"); 

    loop {
        if let Some(quit) = rack.get_output(key, 2) { // "quit" output on keyboard
            if quit > 0.5 {
                break;
            }
        }
        rack.run(Samples(128));
    }
}

