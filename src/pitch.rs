
use std::sync::Arc;
use num_complex::Complex;
use rustfft::*;
use crate::resampler::Resampler;
use crate::units::{Samples, Hz, Cent, Sec, SAMPLE_RATE};
//use crate::module::*;

fn from_db(x: f64) -> f64 { (10.0f64).powf(x / 10.0) }

/*
const THRESH1: f64 = 0.75; // threshold for detection as a factor of r0
const THRESH2: f64 = 24.0; // threshold of autocorr fft peak power, in dB
*/
const THRESH1: f64 = 0.5; // threshold for detection as a factor of r0
const THRESH2: f64 = 12.0; // threshold of autocorr fft peak power, in dB

const MIN_NOTE: f64 = -2.0 * 12.0 * 100.0; // 2 octaves lower than A440
//const MAX_NOTE: f64 = (3.0 * 12.0 + 3.0) * 100.0; // C 3 octaves higher than A440
const MAX_NOTE: f64 = (2.0 * 12.0 + 3.0) * 100.0; // C 2 octaves higher than A440
//const DOWNRATE: usize = 8;
const DOWNRATE: usize = 1;

// Pitch detection by detecting lag that maximizes the autocorrelation
// XXX take FFT of autocorr to find true fundamental
// XXX downsampling increases pitch error since pitch resolution is based on integral number of samples in its period.
// can we use interpolation to get fractional number of samples in the period?
pub struct Pitch {
    pub down_sample: Resampler, // downsample by DOWNRATE 
    pub data: Vec<f64>, // downsampled data
    size: usize, // how much data to collect into a batch
    overlap: usize, // how much data to keep between batches, overlap < size
    pub minscan: SamplesDown,
    pub maxscan: SamplesDown,
    fft: Arc<dyn Fft<f64>>,
    fft_for_fast_autocorr: Arc<dyn Fft<f64>>,

    pub note: Option<Cent>, // the note, if a sufficiently powerful note was detected
    pub power: f64, // power of the fft of the autocorr for the note
    pub corr: f64, // autocorr for the note
    pub fftdata: Vec<Complex<f64>>,
    pub corrdata: Vec<f64>,
}

pub fn autocorr(data: &Vec<f64>, delay: usize) -> f64 {
    let r : f64 = data[0..data.len() - delay]
                .iter()
                .zip(&data[delay..data.len()])
                .map(|(a,b)| a*b)
                .sum();
    r / (data.len()-delay) as f64
}

// compute autocorrelation of data into dst with straightforward method (slow)
fn autocorrs(dst: &mut Vec<f64>, data: &Vec<f64>, minscan: SamplesDown, maxscan: SamplesDown) {
    let mut r0 = autocorr(data, 0);
    if r0 == 0.0 { r0 = 0.000001 };

    for delay in minscan.0 .. maxscan.0 {
        dst[delay] = autocorr(data, delay) / r0;
    }
}

// faster autocorr: take data, zero pad by how many autocorr outputs we care about (k),
// take fft, then multiply each element by its conjugate, then take ifft, and the
// first k elements will have re component with autocorrs (with some small error).
fn fast_autocorrs(dst: &mut Vec<f64>, data: &Vec<f64>, maxscan: SamplesDown, fft: &Arc<dyn Fft<f64>>) {
    let k = maxscan.0;
    let sz = data.len() + k;
    assert!(dst.len() == k);

    // make padded buffer
    let mut fftbuf: Vec<Complex<f64>> = vec![Complex{re: 0.0, im: 0.0}; sz];
    for n in 0..data.len() {
        fftbuf[n] = Complex{ re: data[n], im: 0.0 };
    }

    // approximate autocorr with fft
    fft.process(&mut fftbuf); // forward fft
    fftbuf.iter_mut().for_each(|v| *v = *v * v.conj());
    fft.process(&mut fftbuf); // inverse fft, modulo scaling, which we normalize away anyway

    // normalize results and copy to output
    let inv_r0 = if fftbuf[0].re != 0.0 { 1.0 / fftbuf[0].re } else { 1.0 };
    for n in 0..k {
        dst[n] = fftbuf[n].re * inv_r0;
    }
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

fn note_to_period(note: Cent) -> Sec {
    let Hz(freq) = note.into();
    Sec(1.0 / freq)
}

impl Pitch {
    pub fn new(winsz: impl Into<SamplesDown>, overlap: impl Into<SamplesDown>) -> Self {
        let SamplesDown(winsamples) = winsz.into();
        let SamplesDown(overlapsamples) = overlap.into();
        let mut planner = FftPlanner::new();
        //let maxscan = note_to_period(Cent(MIN_NOTE)).into(); // MIN freq becomes max period
        let maxscan = SamplesDown(winsamples);
        assert!(overlapsamples < winsamples);
        Self {
            down_sample: Resampler::new_down(DOWNRATE),
            data: Vec::new(),
            size: winsamples,
            overlap: overlapsamples,
            fft: planner.plan_fft_forward(winsamples),
            fft_for_fast_autocorr: planner.plan_fft_forward(winsamples + maxscan.0),

            // note: min becomes max and vice versa, because we're converting from freqs to periods
            maxscan: maxscan,
            //minscan: note_to_period(Cent(MAX_NOTE)).into(),
            minscan: SamplesDown(0),
            note: None,
            corr: 0.0,
            power: 0.0,

            fftdata: vec![Complex{ re: 0.0, im: 0.0 }; maxscan.0],
            corrdata: vec![0.0; maxscan.0],
        }
    }

    fn pack_fft_data(&mut self) {
        for n in 0 .. self.maxscan.0 {
            // XXX somewhat wasteful using complex fft on real data series
            self.fftdata[n] = Complex{ re: self.corrdata[n - self.minscan.0], im: 0.0 };
        }
    }

    pub fn add_sample(&mut self, samp: f64) -> bool {
        // XXX we're using a stereo downsampler for mono data, this is wasteful.
        if let Some(downsamp) = self.down_sample.resample_down(samp) {
            if self.data.len() == self.size {
                // shift over the last "overlap" elements to start of vec
                self.data.drain(0 .. self.size - self.overlap);
            }

            self.data.push(downsamp);
        }
        return self.data.len() == self.size;
    }

    fn detect(&mut self) {
        assert!(self.data.len() == self.size);
        //autocorrs(&mut self.corrdata, &self.data, self.minscan, self.maxscan);
        fast_autocorrs(&mut self.corrdata, &self.data, self.maxscan, &self.fft_for_fast_autocorr);
        self.pack_fft_data();
        self.fft.process(&mut self.fftdata);
        let (fftidx,pow) = max_fft(&self.fftdata);

        assert!(self.fftdata.len() == self.corrdata.len());

        // corridx loses some precision due.. perhaps its best to hunt around near corridx for the autocorr local max?
        let corridx = if fftidx > 1 { self.fftdata.len() / fftidx } else { 0 };
        let rdelay = self.corrdata[corridx];
        self.power = pow;
        self.corr = rdelay;
        if fftidx > 1 && pow > from_db(THRESH2) && rdelay > THRESH1 {
            // DOWNSAMPRATE = SAMPRATE/DOWNRATE, ie 48k/8.
            // autocorr indices are in units of downsampled samples, ie. 1.0/DOWNSAMPRATE seconds apart.
            // fft(autocorr) indices are DOWNSAMPRATE/fftdata.len() Hz apart
            let downsamprate = SAMPLE_RATE / DOWNRATE as f64;
            let freqfrac = fftidx as f64 / (self.fftdata.len() as f64);
            let hz = freqfrac * downsamprate;
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

// Samples at slower rate (ie. 1/8th) of the `Samples` type
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Default)]
pub struct SamplesDown(pub usize);

// XXX cant we do these conversions generically?
impl From<Samples> for SamplesDown {
    fn from(s: Samples) -> Self {
        Self(s.0 / DOWNRATE)
    }
}

impl From<SamplesDown> for Samples {
    fn from(s8: SamplesDown) -> Self {
        Self(s8.0 * DOWNRATE)
    }
}

impl From<Sec> for SamplesDown {
    fn from(x: Sec) -> Self {
        let s: Samples = x.into();
        s.into()
    }
}

impl From<SamplesDown> for Sec {
    fn from(x: SamplesDown) -> Self {
        let s: Samples = x.into();
        s.into()
    }
}
