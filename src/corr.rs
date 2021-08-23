
use std::sync::Arc;
use num_complex::Complex;
use num_traits::identities::Zero;
use rustfft::*;

// Efficient in-place normalized autocorrelation computation.
pub struct AutoCorr {
    pub n: usize, // size of input data
    pub k: usize, // number of autocorr values desired

    fft: Arc<dyn Fft<f64>>,
    pub buf: Vec<Complex<f64>>,
}

impl AutoCorr {
    pub fn new(n: usize, k: usize) -> Self {
        let mut planner = FftPlanner::new();
        Self {
            n, k,
            fft: planner.plan_fft_forward(n + k),
            buf: vec![Complex::zero(); n+k],
        }
    }

    // Given buf[0..n] filled with data, compute normalized autocorr into buf[0..k]'s "re" field.
    pub fn process(&mut self) {
        // zeropad buf
        let (n,k) = (self.n, self.k);
        self.buf[n..n+k].iter_mut().for_each(|c| *c = Complex::zero());

        // approximate autocorr with fft
        // XXX we're FFT'ing complexes, but data is real. could we improve perf with real-transforms?
        self.fft.process(&mut self.buf); // forward fft
        self.buf.iter_mut().for_each(|v| *v = *v * v.conj());

        // note: we're using a forward-fft to compute the inverse-fft.
        // this is OK because our resulting signal is real, and we're normalizing
        // the result.
        self.fft.process(&mut self.buf);

        // normalize results and copy to output
        if self.buf[0].re != 0.0 {
            let inv_r0 = 1.0 / self.buf[0].re;
            self.buf[0..k].iter_mut().for_each(|v| v.re *= inv_r0);
        }
    }
}

// Textbook implementation of square-difference function
pub struct NaiveSDF {
    pub n: usize, // input size
    pub k: usize, // output size
    pub buf: Vec<f64>,
}

impl NaiveSDF {
    pub fn new(n: usize, k: usize) -> Self {
        Self { n, k, buf: vec![0.0; k], }
    }

    pub fn process(&mut self, x: &Vec<f64>) {
        assert!(x.len() == self.n);
        for delay in 0..self.k {
            let mut sum = 0.0;
            for j in 0..self.n - delay {
                sum += (x[j] - x[j+delay]).powi(2);
            }
            self.buf[delay] = sum;
        }
    }
}

// Efficient implementation of square-difference function using efficient AutoCorr
pub struct SDF {
    pub n: usize, // input size
    pub k: usize, // output size
    pub buf: Vec<f64>,

    corr: AutoCorr,
}

impl SDF {
    pub fn new(n: usize, k: usize) -> Self {
        Self { 
            n, k,
            buf: vec![0.0; k], 
            corr: AutoCorr::new(n, k),
        }
    }

    pub fn process(&mut self, x: &Vec<f64>) {
        assert!(x.len() == self.n);
        // ref: https://www.cs.otago.ac.nz/research/publications/oucs-2008-03.pdf Sec 3.3.4 (pg 51)
        // let m[d] = SUM (x[j]^2 + x[j+d]^2)
        // SDF[d] = SUM (x[j] - x[j+d])^2
        //        = SUM (x[j]^2 + x[j+d]^2) - 2 * r[d]       ; by expanding (a-b)^2 = a^2 + b^2 - 2 ab
        //        = m[d] - 2 * r[d]
        // note: m[d] can be computed incrementally from m[d-1], starting with m[0] = 2*r[0]
        //   with: m[d] = m[d-1] - x[d-1]^2 - x[N - d]^2
        // note: our corr[d] is normalized by r0, so we have to de-normalize.
        // XXX alternately we could normalize m's by r0.

        // compute r's
        for n in 0 .. x.len() { self.corr.buf[n] = x[n].into(); }
        self.corr.process();

        // re-compute r0 because we lost its true value when normalizing
        let r0: f64 = x.iter().copied().map(|v| v.powi(2)).sum();

        // XXX could compute in-place into self.corr's buffer for space efficiency.
        self.buf[0] = 0.0;
        let mut m = 2.0 * r0;
        for delay in 1..self.k {
            m -= x[delay - 1].powi(2) + x[self.n - delay].powi(2);
            self.buf[delay] = m - 2.0 * r0 * self.corr.buf[delay].re;
        }
    }
}
