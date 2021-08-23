
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

    pub fn process(&mut self, x: &Vec<f64>, norm: bool) {
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
            let sdf = m - 2.0 * r0 * self.corr.buf[delay].re;
            if norm {
                // normalized SDF
                if m != 0.0 {
                    self.buf[delay] = 1.0 - sdf / m;
                } else {
                    self.buf[delay] = 0.0;
                }
            } else {
                // traditional SDF
                self.buf[delay] = sdf;
            }
        }
    }
}

// Given x[k-1], x[k] and x[k+1], estimate the true peak at x[k+d] by fitting to a parabola.
// Returns (d, x_estimated[k+d]).
// Ref: Survey on Extraction of Sinusoids in Stationary Sounds, Keller and Merchand, 2002.
pub fn parabolic_fit_peak_correction(xprev: f64, x: f64, xnext: f64) -> (f64, f64) {
    let d = 0.5 * (xprev - xnext) / (xprev - 2.0 * x + xnext);
    let peak = x - 0.25 * d * (xprev - xnext);
    (d, peak)
}

// Return estimate of the true peak index and value near x[k] by fitting to a parabola.
pub fn parabolic_fit_peak(x: &Vec<f64>, k: usize) -> (f64, f64) {
    if k == 0 || k == x.len() - 1 {
        (k as f64, x[k])
    } else {
        let (d, peak) = parabolic_fit_peak_correction(x[k-1], x[k], x[k+1]);
        (k as f64 + d, peak)
    }
}

