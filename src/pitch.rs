
//use std::f64::consts::PI;
use crate::resampler::Resampler;
use crate::units::{Samples, Hz, Cent, Sec};
//use crate::module::*;

const THRESH: f64 = 0.75; // threshold for detection as a factor of r0
const MIN_NOTE: f64 = -2.0 * 12.0 * 100.0; // 2 octaves lower than A440
//const MAX_NOTE: f64 = (3.0 * 12.0 + 3.0) * 100.0; // C 3 octaves higher than A440
const MAX_NOTE: f64 = (2.0 * 12.0 + 3.0) * 100.0; // C 2 octaves higher than A440
const DOWNRATE: usize = 8;

// Pitch detection by detecting lag that maximizes the autocorrelation
// XXX take FFT of autocorr to find true fundamental
pub struct Pitch {
    pub down_sample: Resampler, // downsample by DOWNRATE 
    pub data: Vec<f64>, // downsampled data
    size: usize, // how much data to collect into a batch
    overlap: usize, // how much data to keep between batches, overlap < size
    pub minscan: SamplesDown,
    pub maxscan: SamplesDown,
    pub note: Option<Cent>, // the note, if a sufficiently powerful note was detected
    pub corr: f64,
}

pub fn autocorr(data: &Vec<f64>, delay: usize) -> f64 {
    let r : f64 = data[0..data.len() - delay]
                .iter()
                .zip(&data[delay..data.len()])
                .map(|(a,b)| a*b)
                .sum();
    r / data.len() as f64
}

fn autocorrs(data: &Vec<f64>, minscan: SamplesDown, maxscan: SamplesDown) -> Vec<f64> {
    let mut r0 = autocorr(data, 0);
    if r0 == 0.0 { r0 = 0.000001 };

    (minscan.0 .. maxscan.0)
        .map(|delay| autocorr(data, delay) / r0)
        .collect()
}

fn max_autocorr(data: &Vec<f64>, minscan: SamplesDown, maxscan: SamplesDown) -> (Option<SamplesDown>, f64) {
    let mut r0 = autocorr(data, 0);
    if r0 == 0.0 { r0 = 0.000001 };

    let mut maxr = 0.0;
    let mut maxdelay = None;
    for delay in minscan.0 .. maxscan.0 {
        let r = autocorr(data, delay);
        if r > maxr {
            maxr = r;
            if maxr > THRESH * r0 {
                maxdelay = Some(SamplesDown(delay));
            }
        }
    }
    (maxdelay, maxr / r0)
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
        assert!(overlapsamples < winsamples);
        Self {
            down_sample: Resampler::new_down(DOWNRATE),
            data: Vec::new(),
            size: winsamples,
            overlap: overlapsamples,
            // note: min becomes max and vice versa, because we're converting from freqs to periods
            maxscan: note_to_period(Cent(MIN_NOTE)).into(),
            minscan: note_to_period(Cent(MAX_NOTE)).into(),
            note: None,
            corr: 0.0,
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

    pub fn autocorrs(&self) -> Vec<f64> {
        assert!(self.data.len() == self.size);
        autocorrs(&self.data, self.minscan, self.maxscan)
    }

    // return detected pitch and the normalized correlation, only when there is a newly computed value
    pub fn proc_sample(&mut self, samp: f64) -> Option<(Option<Cent>, f64)> {
        if self.add_sample(samp) {
            let (optnote, corr) = max_autocorr(&self.data, self.minscan, self.maxscan);

            // convert SamplesDown into Samples and then into a note
            // XXX generic here?
            self.note = optnote.map(period_to_note);
            self.corr = corr;
            Some((self.note, self.corr))
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
