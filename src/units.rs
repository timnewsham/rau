
use std::convert::From;
use std::f64::consts::PI;

// XXX prefer an API where this configurable
// but that would mean that all previous Freqs have to change
// as soon as the sampleRate is changed...  think about this more.

pub const SAMPLE_RATE : f64 = 48000.0; // Hz
pub const MAXRADPS : f64 = PI; // RadPS
pub const MAXHZ : f64 = SAMPLE_RATE / 2.0;

// time in seconds
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct Sec(pub f64);

// time in samples
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct Samples(pub usize);
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct FracSamples(pub usize, pub f64);

impl From<Samples> for Sec {
    fn from(x: Samples) -> Sec {
        Sec(x.0 as f64 / SAMPLE_RATE)
    }
}

impl From<Sec> for Samples {
    fn from(x: Sec) -> Samples {
        Samples((SAMPLE_RATE * x.0) as usize)
    }
}

impl From<Sec> for FracSamples {
    fn from(x: Sec) -> FracSamples {
        let samps = x.0 * SAMPLE_RATE;
        let whole_samps = samps.floor();
        let frac_samps = samps - whole_samps;
        FracSamples(whole_samps as usize, frac_samps)
    }
}

// frequencies in Hz
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct Hz(pub f64);

/*
 * radians per sample.
 * A RadPS of PI represents half the sampling frequency
 * and is the largest frequency representable without aliasing.
 * These units can be used directly as phase velocities
 * to advance a generator's phase between sample times.
 */
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct RadPS(pub f64);

impl From<RadPS> for Hz {
    fn from(x: RadPS) -> Self {
        Hz( x.0 * (SAMPLE_RATE / (2.0 * PI)) )
    }
}

impl From<Hz> for RadPS {
    fn from(hz: Hz) -> Self {
        let mfreq = hz.0 * (2.0 * PI / SAMPLE_RATE);
        debug_assert!(mfreq <= MAXRADPS);
        RadPS(mfreq)
    }
}

/*
 * Cents represent note values in cents above A 440.
 * XXX maybe I should just have "semitones above 440" instead of cents.
 * since its a float anyway...
 * XXX or maybe use midi notes (semitones, with middle C being 60)
 */
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct Cent(pub f64);

impl From<Cent> for Hz {
    fn from(x: Cent) -> Self {
        // hz = 440 * 2^(cent / 1200)
        Hz(440.0 * (2.0_f64).powf(x.0 / 1200.0))
    }
}

impl From<Hz> for Cent {
    fn from(x: Hz) -> Self {
        // cent = 1200 * log2 (hz/440)
        Cent(1200.0 * (x.0 / 440.0).log2())
    }
}

impl From<Cent> for RadPS {
    fn from(x: Cent) -> Self {
        RadPS::from( Hz::from(x) )
    }
}

/*
 * Notes in semitones with middle C being 60.0 and A440 = 69.0
 */
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct MidiNote(pub f64);

impl From<Cent> for MidiNote {
    fn from(x: Cent) -> Self {
        MidiNote(69.0 + x.0 / 100.0)
    }
}

impl From<MidiNote> for Cent {
    fn from(x: MidiNote) -> Self {
        Cent((x.0 - 69.0) * 100.0)
    }
}
