# Audio sound generation

Goofing with audio sound generation in Rust.


# Use

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
