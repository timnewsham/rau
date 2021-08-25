
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

// returns a factor of how much to increase the output frequency.
pub type CorrectFn = fn(Option<Cent>) -> f64;

pub struct PitchCorrect {
    correctfn: CorrectFn,
    pub p: Pitch,
    pub overlap: Vec<f64>, // overlap data from previous window (after repitching)
    pub inputperiod: Option<f64>, // detected period for current window
    inputphase: f64, // phase at the start of the current window of input (before repitching), as frac of periods
    transphase: f64, // phase at the midpoint of the stored overlap data, as frac of periods

    // for module implementation
    inp: f64,
    buf: Vec<f64>,
    pos: usize,
}

// ratio to turn f1 into f2
pub fn freq_ratio(f1: impl Into<Hz>, f2: impl Into<Hz>) -> f64 {
    let Hz(hz1) = f1.into();
    let Hz(hz2) = f2.into();
    hz2 / hz1
}

// Quantize the note (if known) to the nearest 100 cents.
pub fn quantize_note(note: Option<Cent>) -> f64 {
    match note {
        None => 1.0,
        Some(note) => {
            let semitones = note.0 / 100.0;
            let corrected = semitones.round();
            let note2 = Cent(100.0 * corrected);
            freq_ratio(note, note2)
        },
    }
}

const RESAMPORDER: usize = 16;

impl PitchCorrect {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 4 {
            return Err(format!("usage: {} minfreq maxfreq overlap", args[0]));
        }
        let minfreq = parse::<f64>("minfreq", args[1])?;
        let maxfreq = parse::<f64>("maxfreq", args[2])?;
        let overlap = parse::<f64>("overlap", args[3])?;
        Ok( modref_new(Self::new_quantize(Hz(minfreq), Hz(maxfreq), overlap)) )
    }

    pub fn new_quantize(min: impl Into<Cent>, max: impl Into<Cent>, overlapfrac: f64) -> Self {
        Self::new(quantize_note, min, max, overlapfrac)
    }

    pub fn new(correctfn: CorrectFn, min: impl Into<Cent>, max: impl Into<Cent>, overlapfrac: f64) -> Self {
        // we require at least two period of overlap to properly phase match.
        // this meanse overlapfrac should be at least 0.34 for MIN_PERIODS of 6.
        // XXX we should just maintain our own overlap size instead of duplicating pitch's!
        assert!(overlapfrac > 2.0 / MIN_PERIODS as f64);

        let p = Pitch::new(min, max, overlapfrac);
        let overlapsz = p.overlap;
        let outsz = p.size - p.overlap;
        Self { 
            correctfn,
            p,
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
    fn get_outputperiod(&mut self, correction: f64) -> Option<f64> {
        if let Some(p) = self.inputperiod {
            let cp = p / correction;
            if cp.round() != 0.0 {
                return Some(cp);
            }
        }
        return None;
    }

    // Generate outputs that repitch the input frequencies by correction factor.
    pub fn repitch(&mut self, wanted_correction: f64) -> Vec<f64> {
        let (n,m) = resampler::rational_approx(wanted_correction);
        let correction = n as f64 / m as f64;
        let outputperiod = self.get_outputperiod(correction);

        // resample data by inverse of correction to increase frequencies by that amount
        // ie. generate more samples at current rate to decrease the frequency.
        // XXX revisit parameters.  this is intentionally a bit loose assuming lower pitched inputs
        let mut r = resampler::Resampler::new(m, n, 50.0, 0.7, RESAMPORDER);

        // Delay the current outputs by an amount that matches the phase at the midpoint
        // of the overlap with the new data at the midpoint of the overlap.
        let mut data: Vec<f64> = Vec::new();
        let mid_overlap = self.overlap.len() as f64 * 0.5;
        let delay = match outputperiod {
            Some(outperiod) => {
                let midphase = (self.inputphase + mid_overlap / outperiod) % 1.0;
                let phasediff = (1.0 + midphase - self.transphase) % 1.0;
                //println!("midphase {} transphase {}, diff {}", midphase, self.transphase, phasediff);
                //println!("outperiod {}, inperiod {:?}, inputphase {}", outperiod, self.inputperiod, self.inputphase);
                (phasediff * outperiod).round() as usize
            },
            None => 0,
        };
        //println!("delay {}", delay);

        assert!(delay < self.overlap.len());
        for j in 0..delay {
            data.push(self.overlap[j]);
        }

        // fill data[] with resampled data from p.data[pos..] until full, looping as necessasry
        // phase of first input sample is self.inputsample
        // phase of first real output sample (output[delay]) is also self.inputsample
        let n = self.p.data.len();
        let mut inpos = 0;
        let mut inconsumed = 0.0;
        while data.len() < n { // now start capturing data, we're in synch
            r.resample(self.p.data[inpos], |x| if data.len() < n { data.push(x); });
            inpos += 1;
            inconsumed += 1.0;

            // Output needs more samples than input.
            if inpos >= n {
                // if periodic, loop at point with same phase as pos. Otherwise loop from start
                // Excess samples in a fractional period consumed so far since the start of self.p.data[]
                // is `consumed % inputperiod`, which is the same number of samples to skip from start when looping.
                inpos = match self.inputperiod {
                    Some(inper) => (inconsumed % inper).round() as usize,
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
        let numoutputs = (chunk_len - delay) as f64; // samples since starting at inputphase (after delay)
        let transmidpoint = numoutputs + mid_overlap; // samples to transition point since starting at inputphase
        self.transphase = match outputperiod {
            None => 0.0,
            Some(outper) => (self.inputphase + transmidpoint / outper) % 1.0,
        };

        let numinputs = (self.p.size - self.p.overlap) as f64; // new inputs, not counting the overlap
        self.inputphase = match self.inputperiod {
            None => 0.0,
            Some(inper) => (self.inputphase + numinputs / inper) % 1.0,
        };


        data
    }

    // Process next sample, maybe generating a sequence of corrected samples
    pub fn process(&mut self, samp: f64) -> Option<Vec<f64>> {
        if let Some(result) = self.p.proc_sample(samp) {
            self.inputperiod = self.p.period; // XXX perhaps it should be a return value
            let correction = (self.correctfn)(result);
            Some(self.repitch(correction))
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
