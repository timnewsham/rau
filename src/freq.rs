
use std::convert::{From};
use std::f64::consts::PI;

// XXX prefer an API where this configurable
// but that would mean that all previous Freqs have to change
// as soon as the sampleRate is changed...  think about this more.

pub const SAMPLE_RATE : f64 = 44100.0; // Hz
pub const MAXRADPS : f64 = PI; // RadPS

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
        debug_assert!(mfreq <= PI);
        RadPS(mfreq)
    }
}

/*
 * Cents represent note values in cents above A 440.
 * XXX maybe I should just have "semitones above 440" instead of cents.
 * since its a float anyway...
 */
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct Cent(pub f64);

impl From<Cent> for Hz {
    fn from(x: Cent) -> Self {
        Hz(440.0 * (2.0_f64).powf(x.0 / 1200.0))
    }
}

impl From<Cent> for RadPS {
    fn from(x: Cent) -> Self {
        RadPS::from( Hz::from(x) )
    }
}
