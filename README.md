# Audio sound generation

Goofing with audio sound generation in Rust.


# Use

There are two programs that test the current features. 
The [src/bin/keyboard.rs](src/bin/keyboard.rs) example builds a small keyboard-based
synthesizer. Run with `cargo run --release --bin keyboard`.
The [src/bin/rau.rs](src/bin/rau.rs) example tests out various features as I
work on them. Most tests are commented out. Run with
`cargo run`.

Running main with `cargo run` generates some ascii output and
some `.s16` files. The generated files are a single channel of
raw signed 16-bit samples in big endian and can be converted 
with sox or played directly:

    `sox -r 44100 -e signed -B -b 16 -c 1 sweep.s16 sweep.wav`
    `play -r 44100 -e signed -B -b 16 -c 1 sweep.s16`


# Helpers

The `filtviz` subdir contains a frequency response viewer for
the filter parameter generator.  Run with `cargo run -p filtviz`.

The `phaseviz` subdir contains a phase meter. It reads in a file
from the current directory named `test.wav` which should be a
stereo file with 16-bit samples. Run with `cargo run -p phaseviz`.

The `genviz` subdir contains a visualizer for the fourier series
generator. Run with `cargo run -p genviz`.

The `player` subdir contains code to test a resampler implementation,
playing a wav file at 44.1kHz to the audio device at 48kHz.
