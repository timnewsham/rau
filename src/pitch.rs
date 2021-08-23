
use crate::units::{Samples, Hz, Cent, Sec, SAMPLE_RATE};
use crate::corr::{SDF, parabolic_fit_peak};

const PEAK_THRESH: f64 = 0.9; // threshold first peak must be relative to max peak
const CLARITY_THRESH: f64 = 0.80;

// Pitch detection using NSDF to find fundamental.
// ref: https://www.cs.otago.ac.nz/research/publications/oucs-2008-03.pdf
pub struct Pitch {
    pub data: Vec<f64>, 
    size: usize, // how much data to collect into a batch
    overlap: usize, // how much data to keep between batches, overlap < size
    min_note: Cent,

    pub note: Option<Cent>, // the note, if a sufficiently powerful note was detected
    pub clarity: f64, // measure of how strong the note is
    pub nsdf: SDF,

    cnt: usize, // XXX debug helper
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

fn note_to_period(note: Cent) -> Samples {
    let Hz(freq) = note.into();
    let period = Sec(1.0 / freq);
    period.into()
}

const MIN_PERIODS: usize = 6; // bigger gives better accuracy but more latency and computation.

impl Pitch {
    // Pitch detector for range of notes min..max.
    // overlap is a fraction of the total window that we keep between windows.
    pub fn new(min: impl Into<Cent>, max: impl Into<Cent>, overlapfrac: f64) -> Self {
        let min_note: Cent = min.into();
        let max_note: Cent = max.into();
        assert!(0.0 < overlapfrac && overlapfrac < 1.0);
        assert!(min_note < max_note);

        let Samples(max_period) = note_to_period(min_note);
        let size = max_period * MIN_PERIODS;
        let overlap = (size as f64 * overlapfrac) as usize;
        assert!(overlap < size);

        Self {
            data: Vec::new(),
            size, overlap, min_note,

            note: None,
            clarity: 0.0,

            nsdf: SDF::new(size, max_period),

            cnt: 0, // XXX debug
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
            //if self.cnt == 70 { println!("maxdelays {:?}", maxdelays); }
            let peaks = maxdelays[1..].iter().map(|idx| parabolic_fit_peak(&self.nsdf.buf, *idx)).collect();
            let (_, peakval) = max_peak(&peaks);
            let (lag, clarity) = first_peak_above_thresh(&peaks, PEAK_THRESH * peakval);

            // XXX no conversions exist yet for fractional Samples.
            let period = lag / SAMPLE_RATE; // period in seconds

            self.clarity = clarity;
            let note: Cent = Hz(1.0 / period).into();
            if clarity > CLARITY_THRESH && note >= self.min_note {
                self.note = Some(Hz(1.0 / period).into());
            } else {
                self.note = None;
            }
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

