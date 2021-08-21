
//use std::f64::consts::PI;
use crate::units::{Samples, Hz, Cent, Sec};
//use crate::module::*;

const THRESH: f64 = 0.75; // threshold for detection as a factor of r0
const MIN_NOTE: f64 = -2.0 * 12.0 * 100.0; // 2 octaves lower than A440
//const MAX_NOTE: f64 = (3.0 * 12.0 + 3.0) * 100.0; // C 3 octaves higher than A440
const MAX_NOTE: f64 = (2.0 * 12.0 + 3.0) * 100.0; // C 2 octaves higher than A440

// Pitch detection by detecting lag that maximizes the autocorrelation
// XXX use a downsampler for perf. 2^4 * 440 = 7040hz. we can downsample by 6, 48k->8k
pub struct Pitch {
    pub data: Vec<f64>, // XXX use dequeue?
    size: usize,
    overlap: usize, // overlap < size
    minscan: Samples,
    maxscan: Samples,
    pub note: Option<Cent>, // the note, if a sufficiently powerful note was detected
    pub corr: f64,
}

fn autocorr(data: &Vec<f64>, delay: usize) -> f64 {
    let r : f64 = data[0..data.len() - delay]
                .iter()
                .zip(&data[delay..data.len()])
                .map(|(a,b)| a*b)
                .sum();
    r / data.len() as f64
}

fn max_autocorr(data: &Vec<f64>, minscan: Samples, maxscan: Samples) -> (Option<Samples>, f64) {
    let mut r0 = autocorr(data, 0);
    let mut maxr = 0.0;
    let mut maxdelay = None;
    for delay in minscan.0 .. maxscan.0 {
        let r = autocorr(data, delay);
        if r > maxr {
            maxr = r;
            if maxr > THRESH * r0 {
                maxdelay = Some(Samples(delay));
            }
        }
    }
    if r0 == 0.0 { r0 = 0.000001 };
    (maxdelay, maxr / r0)
}

fn period_to_note(period: impl Into<Sec>) -> Cent {
    let Sec(period_secs) = period.into();
    Hz(1.0 / period_secs).into()
}

fn note_to_period(note: Cent) -> Sec {
    let Hz(freq) = note.into();
    Sec(1.0 / freq)
}

impl Pitch {
    pub fn new(winsz: impl Into<Samples>, overlap: impl Into<Samples>) -> Self {
        let Samples(winsamples) = winsz.into();
        let Samples(overlapsamples) = overlap.into();
        assert!(overlapsamples < winsamples);
        Self {
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

    // return detected pitch and the normalized correlation, only when there is a newly computed value
    pub fn add_sample2(&mut self, samp: f64) -> Option<(Option<Cent>, f64)> {
        self.data.push(samp);
        if self.data.len() == self.size {
            let (optnote, corr) = max_autocorr(&self.data, self.minscan, self.maxscan);
            self.note = optnote.map(period_to_note);
            self.corr = corr;

            // shift over the last "overlap" elements to start of vec
            self.data.drain(0 .. self.size - self.overlap);
            Some((self.note, self.corr))
        } else {
            None
        }
    }

    // return the detected pitch, one value per input sample
    pub fn add_sample(&mut self, samp: f64) -> Option<Cent> {
        self.add_sample2(samp);
        self.note
    }
}

