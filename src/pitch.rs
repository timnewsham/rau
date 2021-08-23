
use std::sync::Arc;
use num_complex::Complex;
use num_traits::identities::Zero;
use rustfft::*;
use crate::units::{Samples, Hz, Cent, Sec, SAMPLE_RATE};


const THRESH1: f64 = 0.5; // threshold for detection as a factor of r0
const THRESH2: f64 = 12.0; // threshold of autocorr fft peak power, in dB

fn from_db(x: f64) -> f64 { (10.0f64).powf(x / 10.0) }

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
        self.fft.process(&mut self.buf); // forward fft
        self.buf.iter_mut().for_each(|v| *v = *v * v.conj());
        self.fft.process(&mut self.buf); // inverse fft, modulo scaling, which we normalize away anyway

        // normalize results and copy to output
        if self.buf[0].re != 0.0 {
            let inv_r0 = 1.0 / self.buf[0].re;
            self.buf[0..k].iter_mut().for_each(|v| v.re *= inv_r0);
        }
    }
}

// Pitch detection using fft of autocorrelation to find fundamental.
pub struct Pitch {
    pub data: Vec<f64>, // downsampled data
    size: usize, // how much data to collect into a batch
    overlap: usize, // how much data to keep between batches, overlap < size
    fft: Arc<dyn Fft<f64>>,

    pub note: Option<Cent>, // the note, if a sufficiently powerful note was detected
    pub power: f64, // power of the fft of the autocorr for the note
    pub corr: f64, // autocorr for the note
    pub fftdata: Vec<Complex<f64>>,
    pub corrdata: AutoCorr,
}

// Maximum FFT index and value
pub fn max_fft(fftdata: &Vec<Complex<f64>>) -> (usize, f64) {
    let sz = fftdata.len() / 2;
    let mut idx = 0;
    let mut max = fftdata[0].norm_sqr();
    for n in 0..sz {
        let p = fftdata[n].norm_sqr();
        if p > max {
            idx = n;
            max = p;
        }
    }
    (idx, max)
}

pub fn period_to_note(period: impl Into<Sec>) -> Cent {
    let Sec(period_secs) = period.into();
    Hz(1.0 / period_secs).into()
}

impl Pitch {
    pub fn new(winsz: impl Into<Samples>, overlap: impl Into<Samples>) -> Self {
        let Samples(winsamples) = winsz.into();
        let Samples(overlapsamples) = overlap.into();
        assert!(overlapsamples < winsamples);

        let mut planner = FftPlanner::new();
        Self {
            data: Vec::new(),
            size: winsamples,
            overlap: overlapsamples,
            fft: planner.plan_fft_forward(winsamples),

            note: None,
            corr: 0.0,
            power: 0.0,

            fftdata: vec![Complex{ re: 0.0, im: 0.0 }; winsamples],
            corrdata: AutoCorr::new(winsamples, winsamples),
        }
    }

    pub fn add_sample(&mut self, samp: f64) -> bool {
        if self.data.len() == self.size {
            // shift over the last "overlap" elements to start of vec
            self.data.drain(0 .. self.size - self.overlap);
        }

        self.data.push(samp);
        return self.data.len() == self.size;
    }

    fn detect(&mut self) {
        assert!(self.data.len() == self.size);

        // copy data into corrdata and process it to get autocorrs into self.corrdata.buf[0..k]
        for n in 0..self.data.len() {
            self.corrdata.buf[n] = Complex{ re: self.data[n], im: 0.0 };
        }
        self.corrdata.process();

        // copy autocorrs to fftdata.
        // XXX we could compute fft destructively directly in corrdata.buf for efficiency,
        // but right now pitchviz wants access to the raw corrdata..  future optimization.
        assert!(self.fftdata.len() == self.corrdata.k);
        for n in 0..self.fftdata.len() {
            self.fftdata[n] = self.corrdata.buf[n];
        }

        self.fft.process(&mut self.fftdata);
        let (fftidx,pow) = max_fft(&self.fftdata);

        // corridx loses some precision.. perhaps its best to hunt around near corridx for the autocorr local max?
        let corridx = if fftidx > 1 { self.fftdata.len() / fftidx } else { 0 };
        let rdelay = self.corrdata.buf[corridx].re;
        self.power = pow;
        self.corr = rdelay;
        if fftidx > 1 && pow > from_db(THRESH2) && rdelay > THRESH1 {
            // XXX generic conversion for fft index into frequencies and notes?
            let freqfrac = fftidx as f64 / (self.fftdata.len() as f64);
            let hz = freqfrac * SAMPLE_RATE;
            self.note = Some(Hz(hz).into());
        } else {
            self.note = None;
        }
    }

    // return detected pitch if we've processed enough data
    // if not None, fftdata, corrdata, power, note, etc. have been updated and are available
    pub fn proc_sample(&mut self, samp: f64) -> Option<Option<Cent>> {
        if self.add_sample(samp) {
            self.detect();
            Some(self.note)
        } else {
            None
        }
    }

    // return the detected pitch, one value per input sample
    pub fn sample_to_note(&mut self, samp: f64) -> Option<Cent> {
        self.proc_sample(samp);
        self.note
    }
}

