/*
 * Load a synth from a config file and run it.
 */

use std::env;
use rau::loader;
use rau::units::Samples;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut l = loader::Loader::new();

    if args.len() != 2 {
        println!("usage: {} fname", args[0]);
        println!("modules:");
        l.show_usage();
        return;
    }
    let fname = &args[1];

    match l.load(fname) {
        Err(e) => println!("{}", e),
        Ok(mut rack) => {
            while rack.run(Samples(128)) {
                continue;
            }
        },
    };
}
