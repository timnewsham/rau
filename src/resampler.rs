
use std::f64::consts::PI;
pub use crate::speaker::Sample;

// single-channel resampler
pub struct Resampler {
    n: usize,
    m: usize,
    phasefilt: Vec<Vec<f64>>,
    delay: Vec<f64>,
    delaypos: usize,
    order: usize,
    phase: usize,
}

impl Resampler {
    // resample 44100->48000 (160/147 ratio), LP 70dB down with cutoff 80% of 22050 (17640Hz), 32-order FIR
    pub fn new_441_to_480() -> Self {
        Self::new(160, 147, 70.0, 0.8, 32)
    }

    // downsample by 1/down with 70dB down, cutoff at 80% of new sampling rate, 32-order FIR.
    pub fn new_down(down: usize) -> Self {
        Self::new(1, down, 70.0, 0.8, 32)
    }

    pub fn new(n: usize, m: usize, atten: f64, cutoff: f64, order: usize) -> Self {
        let filt = make_fir(n, atten, cutoff, order);
        Self {
            n,
            m,
            phasefilt: filt,
            delay: vec![0.0; order],
            delaypos: 0,
            order,
            phase: 0,
        }
    }

    // resample down, generating at most a single sample for each input
    pub fn resample_down(&mut self, sample: f64) -> Option<f64> {
        assert!(self.n <= self.m);
        let mut out = None;
        self.resample(sample, |s| out = Some(s));
        out
    }

    pub fn resample<F: FnMut(f64)>(&mut self, sample: f64, mut cb: F) {
        // add new sample to delay line
        self.delay[self.delaypos] = sample;

        // generate output samples
        while self.phase < self.n {
            let filt = &self.phasefilt[self.phase];
            self.phase += self.m;

            // convolution with selected phase filter
            let mut out = 0.0;
            let mut pos = self.delaypos;
            for coef in filt.iter() {
                out += self.delay[pos] * coef;
                pos += 1;
                if pos == self.order {
                    pos = 0;
                }
            }
            cb(out)
        }

        self.phase -= self.n;
        self.delaypos = if self.delaypos == 0 {
                self.order - 1
            } else {
                self.delaypos - 1
            };
    }
}

// Two-channel resampler. 
// XXX lots of duplicate code here.
// XXX why not make a generic multi-channel resampler?
pub struct ResamplerStereo {
    n: usize,
    m: usize,
    phasefilt: Vec<Vec<f64>>,
    delayl: Vec<f64>,
    delayr: Vec<f64>,
    delaypos: usize,
    order: usize,
    phase: usize,
}

impl ResamplerStereo {
    // resample 44100->48000 (160/147 ratio), LP 70dB down with cutoff 80% of 22050 (17640Hz), 32-order FIR
    pub fn new_441_to_480() -> Self {
        Self::new(160, 147, 70.0, 0.8, 32)
    }

    // downsample by 1/down with 70dB down, cutoff at 80% of new sampling rate, 32-order FIR.
    pub fn new_down(down: usize) -> Self {
        Self::new(1, down, 70.0, 0.8, 32)
    }

    pub fn new(n: usize, m: usize, atten: f64, cutoff: f64, order: usize) -> Self {
        let filt = make_fir(n, atten, cutoff, order);
        Self {
            n,
            m,
            phasefilt: filt,
            delayl: vec![0.0; order],
            delayr: vec![0.0; order],
            delaypos: 0,
            order,
            phase: 0,
        }
    }

    // resample down, generating at most a single sample for each input
    pub fn resample_down(&mut self, sample: Sample) -> Option<Sample> {
        assert!(self.n <= self.m);
        let mut out = None;
        self.resample(sample, |s| out = Some(s));
        out
    }

    pub fn resample<F: FnMut(Sample)>(&mut self, sample: Sample, mut cb: F) {
        // add new sample to delay line
        self.delayl[self.delaypos] = sample.left;
        self.delayr[self.delaypos] = sample.right;

        // generate output samples
        while self.phase < self.n {
            let filt = &self.phasefilt[self.phase];
            self.phase += self.m;

            // convolution with selected phase filter
            let mut outl = 0.0;
            let mut outr = 0.0;
            let mut pos = self.delaypos;
            for coef in filt.iter() {
                outl += self.delayl[pos] * coef;
                outr += self.delayr[pos] * coef;
                pos += 1;
                if pos == self.order {
                    pos = 0;
                }
            }
            cb(Sample{ left: outl, right: outr })
        }

        self.phase -= self.n;
        self.delaypos = if self.delaypos == 0 {
                self.order - 1
            } else {
                self.delaypos - 1
            };
    }
}

fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-9 { 1.0 } else { x.sin() / x }
}

// n is upsampling factor, m is downsampling factor.
// atten is filter band attenuation in dB (around 70).
// cutoff is a fraction of the original nyquist frequency (like 0.9)
// order is the number of filter coefficients evaluated per output (like 32)
fn make_fir(n: usize, atten: f64, cutoff: f64, order: usize) -> Vec<Vec<f64>> {
    // generate FIR coefficients as windowed sinc
    let wc = PI * cutoff / (n as f64);
    let alpha = -325.1e-6 * atten*atten + 0.1677 * atten - 3.149;
    let fullorder = order * n;
    let mid = (fullorder as f64 - 1.0) / 2.0;
    let filt: Vec<f64> = (0..fullorder).map(|k| {
            // cosh window: https://dsp.stackexchange.com/questions/37714/kaiser-window-approximation
            let x = k as f64 - mid;
            let normx = 2.0 * x / (fullorder as f64);
            let win = ((1.0 - normx*normx).sqrt() * alpha).cosh() / alpha.cosh();
            sinc(x * wc) * win
        }).collect();

    // distribute coefficients into N phase filters
    let mut phasefilt: Vec<Vec<f64>> = (0..n).map(|_| Vec::new()).collect();
    for (k, coeff) in filt.iter().enumerate() {
        phasefilt[k % n].push(*coeff);
    }
    phasefilt
}

