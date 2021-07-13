
use std::f64::consts::PI;

// XXX prefer an API where this configurable
// but that would mean that all previous Freqs have to change
// as soon as the sampleRate is changed...  think about this more.

pub const SAMPLE_RATE : f64 = 44100.0;

pub const MAXFREQ : f64 = PI; // in rad/sample

/*
 * Freqs represent frequencies in radians per sample.
 * A frequency of PI represents half the sampling frequency
 * and is the largest frequency representable without aliasing.
 * These frequencies can be used directly as phase velocities
 * to advance a generator's phase between sample times.
 */
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct Freq(pub f64);

impl Freq {
    // Get internal frequence from Hz
    pub fn from_hz(hz: f64) -> Freq {
        let mfreq = hz * (2.0 * PI / SAMPLE_RATE);
        debug_assert!(mfreq <= PI);
        Freq( mfreq )
    }
}

/*
 * Cents represent note values in cents above A 440.
 * XXX maybe I should just have "semitones above 440" instead of cents.
 * since its a float anyway...
 */
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct Cent(pub f64);

impl Cent {
    pub fn new(cent: f64) -> Cent {
        Cent(cent)
    }

    // Get internal frequence from Hz
    pub fn from_hz(hz: f64) -> Freq {
        let mfreq = hz * (2.0 * PI / SAMPLE_RATE);
        debug_assert!(mfreq <= PI);
        Freq( mfreq )
    }

    pub fn to_hz(self) -> f64 {
        440.0 * (2.0_f64).powf(self.0 / 1200.0)
    }

    pub fn to_freq(self) -> Freq {
        Freq::from_hz(self.to_hz())
    }
}

