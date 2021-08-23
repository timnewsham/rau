
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

