
use crate::units::{Samples, Hz, Cent, Sec, SAMPLE_RATE};
use crate::resampler;
use crate::corr::{SDF, parabolic_fit_peak};
use crate::module::*;

const PEAK_THRESH: f64 = 0.9; // threshold first peak must be relative to max peak
const CLARITY_THRESH: f64 = 0.80;

// Pitch detection using NSDF to find fundamental.
// ref: https://www.cs.otago.ac.nz/research/publications/oucs-2008-03.pdf
pub struct Pitch {
    pub data: Vec<f64>, 
    pub size: usize, // how much data to collect into a batch
    pub overlap: usize, // how much data to keep between batches, overlap < size
    min_note: Cent,

    pub period: Option<f64>, // detected period in samples whenever possible
    pub note: Option<Cent>, // the note, if a sufficiently powerful note was detected
    pub clarity: f64, // measure of how strong the note is
    pub nsdf: SDF,
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
            period: None,

            nsdf: SDF::new(size, max_period),
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

        // compute NSDF into sdf.buf[0..k]
        self.nsdf.process(&self.data, true);

        // find all candidate peaks between zero crossings
        // then find the first one that is within THRESH of the biggest one.
        let maxdelays = maxes(&self.nsdf.buf[0 .. self.nsdf.k]);
        if maxdelays.len() > 1 {
            let peaks = maxdelays[1..].iter().map(|idx| parabolic_fit_peak(&self.nsdf.buf, *idx)).collect();
            let (_, peakval) = max_peak(&peaks);
            let (lag, clarity) = first_peak_above_thresh(&peaks, PEAK_THRESH * peakval);

            // XXX no conversions exist yet for fractional Samples. so we hard code the conversion here
            let period = lag / SAMPLE_RATE; // period in seconds

            self.clarity = clarity;
            let note: Cent = Hz(1.0 / period).into();
            if note >= self.min_note {
                // store best guess for period, in samples, as long as its in range, even if we don't report a note.
                // Any good guess for the period is useful when phase matching samples.
                self.period = Some(lag);
            } else {
                self.period = None;
            }
            if clarity > CLARITY_THRESH && note >= self.min_note {
                self.note = Some(Hz(1.0 / period).into());
            } else {
                self.note = None;
            }
        } else {
            self.note = None;
            self.clarity = 0.0;
            self.period = None;
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

pub struct PitchCorrect {
    p: Pitch,
    correction: f64,
    overlap: Vec<f64>, // overlap data from previous window (after repitching)
    inputphase: f64, // phase at the start of the current window of input (before repitching)
    inputperiod: Option<f64>, // detected period for current window
    transphase: f64, // phase at the midpoint of the stored overlap data

    // for module implementation
    inp: f64,
    buf: Vec<f64>,
    pos: usize,
}

// correct the note. XXX this should be a configurable parameter of the corrector
fn correct(note: Cent) -> Cent {
    let semitones = note.0 / 100.0;
    let corrected = semitones.round();
    Cent(100.0 * corrected)
}

impl PitchCorrect {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 4 {
            return Err(format!("usage: {} minfreq maxfreq overlap", args[0]));
        }
        let minfreq = parse::<f64>("minfreq", args[1])?;
        let maxfreq = parse::<f64>("maxfreq", args[2])?;
        let overlap = parse::<f64>("order", args[3])?;
        Ok( modref_new(Self::new(Hz(minfreq), Hz(maxfreq), overlap)) )
    }

    pub fn new(min: impl Into<Cent>, max: impl Into<Cent>, overlapfrac: f64) -> Self {
        // we require at least one period of overlap to properly phase match.
        // this meanse overlapfrac should be at least 0.17 for MIN_PERIODS of 6.
        assert!(overlapfrac > 1.0 / MIN_PERIODS as f64);

        let p = Pitch::new(min, max, overlapfrac);
        let overlapsz = p.overlap;
        let outsz = p.size - p.overlap;
        Self { 
            p,
            correction: 1.0,
            overlap: vec![0.0; overlapsz],
            inputphase: 0.0,
            transphase: 0.0,
            inputperiod: None,

            inp: 0.0,
            buf: vec![0.0; outsz],
            pos: 0,
        }
    }

    // Return the corrected period, if its in range.
    // If its out of range then mark the input period as out of range, too.
    fn get_corrected_period(&mut self) -> Option<f64> {
        if let Some(p) = self.inputperiod {
            let cp = self.correction * p;
            if cp.round() != 0.0 {
                return Some(cp);
            }
        }
        return None;
    }

    // We have inputs and a correction factor, generate new outputs.
    pub fn repitch(&mut self) -> Vec<f64> {
        let corrected_period = self.get_corrected_period();

        // Delay the current window by an amount that matches the phase at the midpoint
        // of the overlap with the new data at position (0.5*overlapsize - delay).
        let mut data: Vec<f64> = Vec::new();
        let mid_overlap = self.overlap.len() as f64 * 0.5;
        let delay = match corrected_period {
            Some(period) => {
                let midphase = self.inputphase + mid_overlap / period;
                let frac_period = (midphase + 1.0 - self.transphase) % 1.0;
                (frac_period * period) as usize
            },
            None => 0,
        };

        assert!(delay < self.overlap.len());
        let mut pos = 0;
        while pos < delay {
            data.push(self.overlap[pos]);
            pos += 1;
        }

        // create buf of resampled data from p.data
        // XXX revisit parameters.  this is intentionally a bit loose assuming lower pitched inputs
        let mut r = resampler::Resampler::new_approx(self.correction, 50.0, 0.7, 16);
        let n = self.p.data.len();
        assert!(delay < n);
        while data.len() < n {
            r.resample(self.p.data[pos], |x| if data.len() < n { data.push(x); });
            pos += 1;

            // Output needs more samples than input.
            if pos >= n {
                // if periodic, loop at point with same phase as pos. Otherwise loop from start
                pos = match corrected_period {
                    Some(cper) => pos % (cper.round() as usize),
                    None => 0,
                }
            }
        }

        // mix in the previous overlap with a linear fade
        let m = self.overlap.len();
        for n in 0 .. m {
            let alpha = n as f64 / m as f64;
            // fade out self.overlap[] and fade in data[]
            data[n] = (1.0 - alpha) * self.overlap[n] + alpha * data[n];
        }

        // save the end as the next overlap, and return the prefix before that overlap.
        let chunk_len = data.len() - self.overlap.len();
        self.overlap.copy_from_slice(&data[chunk_len ..]);
        data.truncate(chunk_len);

        // update phases
        let startpoint = chunk_len as f64;
        let transmidpoint = startpoint + mid_overlap;
        self.inputphase = match self.inputperiod {
            None => 0.0,
            Some(inper) => (self.inputphase + startpoint / inper as f64) % 1.0,
        };
        self.transphase = match corrected_period {
            None => 0.0,
            Some(cper) => (self.inputphase + transmidpoint / cper) % 1.0,
        };

        data
    }

    // Process next sample, maybe generating a sequence of corrected samples
    pub fn process(&mut self, samp: f64) -> Option<Vec<f64>> {
        if let Some(result) = self.p.proc_sample(samp) {
            if let Some(note) = result {
                let note2 = correct(note);
                let Hz(f1) = note.into();
                let Hz(f2) = note2.into();
                self.correction = f2 / f1;
            } else {
                self.correction = 1.0;
            }
            self.inputperiod = self.p.period; // XXX perhaps it should be a return value
            Some(self.repitch())
        } else {
            None
        }
    } 

    pub fn advance(&mut self) {
        if let Some(buf) = self.process(self.inp) {
            assert!(buf.len() == self.buf.len());
            self.buf.copy_from_slice(&buf);
            self.pos = 0;
        } else {
            self.pos += 1;
            if self.pos >= self.buf.len() { self.pos = 0; }
        }
    }
}

impl Module for PitchCorrect {
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        (vec!["in".to_string()],
         vec!["out".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 { return Some(self.buf[self.pos]); }
        None
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 { self.inp = value; }
    }

    fn advance(&mut self) -> bool {
        self.advance();
        true
    }
}

pub fn init(l: &mut Loader) {
    l.register("pitchcorrect", PitchCorrect::from_cmd);
}
