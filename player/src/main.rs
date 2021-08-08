
use std::env::args;
use rau::speaker::ResamplingSpeaker;
use rau::wav::read_wav;

fn main() {
    let args: Vec<String> = args().collect();
    let path = if args.len() > 1 { &args[1] } else { "test.wav" };
    let mut au = ResamplingSpeaker::new_441_to_480(128);
    let samples = read_wav(path, 44100.0);
    samples.iter().for_each(|s| au.play(*s));
}
