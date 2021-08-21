
use std::f64::consts::PI;
use crate::units::{Samples, Hz, Cent, Sec};
//use crate::module::*;

const THRESH: f64 = 0.7; // threshold for detection as a factor of r0
const MIN_NOTE: f64 = -2.0 * 12.0 * 100.0; // 2 octaves lower than A440
//const MAX_NOTE: f64 = (3.0 * 12.0 + 3.0) * 100.0; // C 3 octaves higher than A440
const MAX_NOTE: f64 = (2.0 * 12.0 + 3.0) * 100.0; // C 2 octaves higher than A440

// Pitch detection by detecting lag that maximizes the autocorrelation
pub struct Pitch {
    pub data: Vec<f64>, // XXX use dequeue?
    pub window: Vec<f64>,
    size: usize,
    overlap: usize, // overlap < size
    minscan: Samples,
    maxscan: Samples,
    pub note: Option<Cent>, // the note, if a sufficiently powerful note was detected
}

fn make_window(n: usize) -> Vec<f64> {
    // using a simple sine window. the shape shouldnt be that important...
    (1..n+1)
        .map(|k| (PI * k as f64 / (n+1) as f64).sin())
        .collect()
}

fn window(data: &Vec<f64>, window: &Vec<f64>) -> Vec<f64> {
    data.iter()
        .zip(window)
        .map(|(a,b)| a*b)
        .collect()
}

fn autocorr(data: &Vec<f64>, delay: usize) -> f64 {
    data[0..data.len() - delay]
        .iter()
        .zip(&data[delay..data.len()])
        .map(|(a,b)| a*b)
        .sum()
}

fn max_autocorr(data: &Vec<f64>, minscan: Samples, maxscan: Samples) -> Option<Samples> {
    let mut maxr = THRESH * autocorr(data, 0);
    //println!("maxr {}", maxr);
    let mut maxdelay = None;
    for delay in minscan.0 .. maxscan.0 {
        let r = autocorr(data, delay);
        if r > maxr {
            maxr = r;
            maxdelay = Some(Samples(delay));
        }
    }
    maxdelay
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
        //println!("size {} overlap {}", winsamples, overlapsamples);
        Self {
            data: Vec::new(),
            window: make_window(winsamples),
            size: winsamples,
            overlap: overlapsamples,
            // note: min becomes max and vice versa, because we're converting from freqs to periods
            maxscan: note_to_period(Cent(MIN_NOTE)).into(),
            minscan: note_to_period(Cent(MAX_NOTE)).into(),
            note: None,
        }
    }

    pub fn add_sample(&mut self, samp: f64) -> Option<Cent> {
        self.data.push(samp);
        if self.data.len() == self.size {
            //println!("scan {:?} to {:?}", self.minscan, self.maxscan);
            let windat = window(&self.data, &self.window);
            self.note = max_autocorr(&windat, self.minscan, self.maxscan).map(period_to_note);

            // shift over the last "overlap" elements to start of vec
            self.data.drain(0 .. self.size - self.overlap);
        }
        self.note
    }
}

