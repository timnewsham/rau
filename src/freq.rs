
use std::f64::consts::PI;

// XXX prefer an API where this configurable
// but that would mean that all previous Freqs have to change
// as soon as the sampleRate is changed...  think about this more.

//pub const SAMPLE_RATE : f64 = 44100.0;
pub const SAMPLE_RATE : f64 = 40.0; // XXX 40 lines of ascii output

/*
 * Freq()s represent frequencies in radians, scaled to the sampling
 * frequency. A frequency of PI represents half the sampling frequency.
 * All real frequencies should be between 0 and PI and 
 * all phases should be between 0 and 2*PI.
 *
 * These frequencies are natural for the sampling frequency, which
 * means sample number n of a sine signal would just be
 * sin(freq.0 * n).  So these values can be used as a phase advance.
 */
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct Freq(f64);

impl Freq {
    // Get internal frequence from Hz
    pub fn from_hz(hz: f64) -> Freq {
        let mfreq = hz * (2.0 * PI / SAMPLE_RATE);
        debug_assert!(mfreq <= PI);
        Freq( mfreq )
    }

    // Get the harmonic frequency, if its representable at our
    // sampling frequency.
    pub fn harmonic(self, n: u64) -> Option<Freq> {
        let mfreq = self.0 * n as f64;
        if mfreq <= PI {
            Some(Freq(mfreq))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PhaseAccum {
    accum : Freq,
    adv : Freq,
}

impl PhaseAccum {
    pub fn from_freq(freq: Freq) -> PhaseAccum {
        PhaseAccum {
            accum: Freq::default(),
            adv: freq,
        }
    }

    pub fn from_hz(hz: f64) -> PhaseAccum {
        PhaseAccum::from_freq(Freq::from_hz(hz))
    }

    // Advance a phase accumulator by one step for the given
    // frequency, keeping the resulting number in range [0, 2*PI]
    pub fn advance(&mut self) {
        self.accum.0 = (self.accum.0 + self.adv.0) % (2.0 * PI);
    }

    pub fn sin(self) -> f64 {
        return self.accum.0.sin();
    }
}

// An additive generator generates a signal as a sum of SIN waves.
pub struct AddGenerator {
    series: Vec<(f64, PhaseAccum)>,
}

fn neg1_k(k: f64) -> f64 {
    f64::powf(-1.0, k)
}

impl AddGenerator {
    /*
    pub fn new_sin(hz: f64) -> AddGenerator {
        AddGenerator{
            series: vec![ (1.0, PhaseAccum::from_hz(hz)) ],
        }
    }
    */

    // XXX this API will have us create a new generator each time
    // we want to change frequencies or waveforms.  It would be
    // preferable to update the weights without discarding the phases.
    //
    // This could cause problems when we have overtones that
    // exceed sampfreq/2 and cant be represented.  How would we
    // maintain their phases when the frequency changes and then
    // drops back in range again?  Perhaps every freq change warrants
    // a phase reset?
    pub fn new_fourier(hz: f64, weights: Vec<(u64, f64)>) -> AddGenerator {
        let freq = Freq::from_hz(hz);
        let mut series = Vec::new();
        for (k, weight) in weights.iter() {
            match freq.harmonic(*k) {
            Some(adv) => series.push((*weight, PhaseAccum::from_freq(adv))),
            None => (),
            }
        }
        AddGenerator{
            series: series,
        }
    }
    pub fn new_sin(hz: f64) -> AddGenerator {
        AddGenerator::new_fourier(hz, vec![(1, 1.0)])
    }

    // sawtooth up
    pub fn new_saw(hz: f64, n: u64) -> AddGenerator {
        debug_assert!(0 < n && n < 100);
        let mut weights = Vec::new();
        for k in 1..=n {
            let kf = k as f64;
            weights.push((k, -2.0 * neg1_k(kf) /(kf * PI)));
        }
        AddGenerator::new_fourier(hz, weights)
    }

    pub fn new_triangle(hz: f64, n: u64) -> AddGenerator {
        debug_assert!(0 < n && n < 100);
        let mut weights = Vec::new();
        for kk in 1..=n {
            let k = 2*kk - 1; // odd harmonics only
            let kf = k as f64;
            let w = 8.0 * neg1_k((kf-1.0)/2.0) / ((kf * PI).powf(2.0));
            weights.push((k, w));
        }
        AddGenerator::new_fourier(hz, weights)
    }

    pub fn new_square(hz: f64, n: u64) -> AddGenerator {
        debug_assert!(0 < n && n < 100);
        let mut weights = Vec::new();
        for kk in 1..=n {
            let k = 2*kk - 1; // odd harmonics only
            let kf = k as f64;
            let w = -4.0 * neg1_k(kf) / (kf * PI);
            weights.push((k, w));
        }
        AddGenerator::new_fourier(hz, weights)
    }

    pub fn advance(&mut self) {
        for (_, accum) in self.series.iter_mut() {
            accum.advance();
        }
    }

    pub fn gen(&mut self) -> f64 {
        let mut x = 0.0;
        for (b, accum) in self.series.iter() {
            x += b * accum.sin();
        }
        return x;
    }
}


