
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
pub struct Freq(f64);

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
 */
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct Cent(f64);

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



// Param in a harmonic series
// note: k is usually but need not be an integer.
struct HarmonicParam {
    k: f64,
    amp: f64,
}

// Easier name for (-1)^k
fn neg1_k(k: f64) -> f64 {
    f64::powf(-1.0, k)
}

fn sine_series() -> Vec<HarmonicParam> {
    vec![HarmonicParam{ k: 1.0, amp: 1.0 }]
}

fn saw_up_series(n: u64) -> Vec<HarmonicParam> {
    debug_assert!(0 < n && n < 100);
    let mut weights = Vec::new();
    for k in 1..=n {
        let kf = k as f64;
        let w = -2.0 * neg1_k(kf) /(kf * PI);
        weights.push(HarmonicParam{ k: k as f64, amp: w });
    }
    weights
}

fn triangle_series(n: u64) -> Vec<HarmonicParam> {
    let mut weights = Vec::new();
    for kk in 1..=n {
        let k = 2*kk - 1; // odd harmonics only
        let kf = k as f64;
        let w = 8.0 * neg1_k((kf-1.0)/2.0) / ((kf * PI).powf(2.0));
        weights.push(HarmonicParam{ k: k as f64, amp: w });
    }
    weights
}

fn square_series(n: u64) -> Vec<HarmonicParam> {
    debug_assert!(0 < n && n < 100);
    let mut weights = Vec::new();
    for kk in 1..=n {
        let k = 2*kk - 1; // odd harmonics only
        let kf = k as f64;
        let w = -4.0 * neg1_k(kf) / (kf * PI);
        weights.push(HarmonicParam{ k: k as f64, amp: w });
    }
    weights
}


// An additive generator generates a signal as a sum of SIN waves.
pub struct HarmonicGenerator {
    // invariant: 0 <= phase < 2*PI
    phase: Freq,

    // invariant: 0 <= velocity < PI
    velocity: Freq,

    series: Vec<HarmonicParam>,
}

impl HarmonicGenerator {
    // internal constructor
    fn new_series(hz: f64, series: Vec<HarmonicParam>) -> HarmonicGenerator {
        // XXX truncate series to prevent aliasing
        HarmonicGenerator {
            phase: Freq::default(),
            velocity: Freq::from_hz(hz),
            series: series,
        }
    }

    pub fn new_sine(hz: f64) -> HarmonicGenerator {
        HarmonicGenerator::new_series(hz, sine_series())
    }

    pub fn new_triangle(hz: f64, n: u64) -> HarmonicGenerator {
        HarmonicGenerator::new_series(hz, triangle_series(n))
    }

    pub fn new_saw_up(hz: f64, n: u64) -> HarmonicGenerator {
        HarmonicGenerator::new_series(hz, saw_up_series(n))
    }

    pub fn new_square(hz: f64, n: u64) -> HarmonicGenerator {
        HarmonicGenerator::new_series(hz, square_series(n))
    }

    pub fn set_freq(&mut self, hz: f64) {
        self.velocity = Freq::from_hz(hz);
    }

    pub fn set_phase(&mut self, theta: f64) {
        debug_assert!(theta >= 0.0);
        self.phase.0 = theta % (2.0 * PI);
    }

    pub fn set_sine(&mut self) {
        self.series = sine_series();
    }

    pub fn set_triangle(&mut self, n: u64) {
        self.series = triangle_series(n);
    }

    pub fn set_saw_up(&mut self, n: u64) {
        self.series = saw_up_series(n);
    }

    pub fn set_square(&mut self, n: u64) {
        self.series = square_series(n);
    }

    pub fn advance(&mut self) {
        self.phase.0 = (self.phase.0 + self.velocity.0) % (2.0 * PI);
    }

    pub fn gen(&mut self) -> f64 {
        let mut x = 0.0;
        for param in self.series.iter() {
            // disallow aliasing
            // XXX would be better to trim series once instead of each gen
            if param.k * self.velocity.0 < MAXFREQ {
                x += param.amp * (param.k * self.phase.0).sin();
            }
        }
        x
    }

    pub fn cost(&self) -> usize {
        let mut cost = 0;
        for param in self.series.iter() {
            if param.k * self.velocity.0 < MAXFREQ {
                cost += 1;
            }
        }
        cost
    }
}


