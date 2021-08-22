
use std::env::args;
use rau::speaker::{player_at};
use rau::wav::read_wav;

fn main() {
    let args: Vec<String> = args().collect();
    let path = if args.len() > 1 { &args[1] } else { "test.wav" };
    let (fsamp, samples) = read_wav(path);
    let mut au = player_at(fsamp, 1000);
    samples.iter().for_each(|s| au.play(*s));
}
