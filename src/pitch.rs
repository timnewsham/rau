
use crate::units::{Samples, Hz, Cent, Sec, SAMPLE_RATE};
use crate::corr::{SDF, parabolic_fit_peak};

const THRESH: f64 = 0.9; // threshold first peak must be relative to max peak

// Pitch detection using NSDF to find fundamental.
// ref: https://www.cs.otago.ac.nz/research/publications/oucs-2008-03.pdf
pub struct Pitch {
    pub data: Vec<f64>, 
    size: usize, // how much data to collect into a batch
    overlap: usize, // how much data to keep between batches, overlap < size

    pub note: Option<Cent>, // the note, if a sufficiently powerful note was detected
    pub clarity: f64, // measure of how strong the note is
    pub nsdf: SDF,

    cnt: usize,
}

pub fn period_to_note(period: impl Into<Sec>) -> Cent {
    let Sec(period_secs) = period.into();
    Hz(1.0 / period_secs).into()
}

// Return the position of all maxima between zero crossings
fn maxes(data: &[f64]) -> Vec<usize> {
    let mut idxs : Vec<usize> = Vec::new();
    let mut max = 0.0;
    let mut maxidx = 0;
    for (n, v) in data.iter().enumerate() {
        if *v < 0.0 && max > 0.0 {
            // downward zero crossing, capture latest maximum
            idxs.push(maxidx);
            max = 0.0;
        }
        if *v > max {
            max = *v;
            maxidx = n;
        }
    }

    // XXX we might want to discard this one anyway...
    if max > 0.0 {
        // maximum between latest zero crossing and end of data
        idxs.push(maxidx);
    }

    idxs
}

fn max_peak(peaks: &Vec<(f64, f64)>) -> (f64, f64) {
    let mut max = peaks[0];
    for peak in peaks.iter() {
        if peak.1 > max.1 {
            max = *peak;
        }
    }
    max
}

fn first_peak_above_thresh(peaks: &Vec<(f64, f64)>, thresh: f64) -> (f64, f64) {
    for peak in peaks.iter() {
        if peak.1 > thresh { return *peak; }
    }
    unreachable!();
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

            note: None,
            clarity: 0.0,

            nsdf: SDF::new(winsamples, winsamples), // XXX k can be smaller, based on min pitch to detect

            cnt: 0,
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
        self.cnt += 1; // XXX
        assert!(self.data.len() == self.size);

        // compute NSDF into sdf.buf[0..k]
        self.nsdf.process(&self.data, true);

        // find all candidate peaks between zero crossings
        // then find the first one that is within THRESH of the biggest one.
        let maxdelays = maxes(&self.nsdf.buf[0 .. self.nsdf.k]);
        if maxdelays.len() > 1 {
if self.cnt == 70 { println!("maxdelays {:?}", maxdelays); }
            let peaks = maxdelays[1..].iter().map(|idx| parabolic_fit_peak(&self.nsdf.buf, *idx)).collect();
if self.cnt == 70 { println!("peaks {:?}", peaks); }
            let (_, peakval) = max_peak(&peaks);
if self.cnt == 70 { println!("peakval {:?}", peakval); }
            let (lag, clarity) = first_peak_above_thresh(&peaks, THRESH * peakval);
if self.cnt == 70 { println!("lag {:?} clarity {:?}", lag, clarity); }

            // XXX no conversions exist yet for fractional Samples.
            let period = lag / SAMPLE_RATE; // period in seconds

            // XXX some thresholding for returning None as a note
            self.note = Some(Hz(1.0 / period).into());
            self.clarity = clarity;
        } else {
            self.note = None;
            self.clarity = 0.0;
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

