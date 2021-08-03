
use std::f32::consts::PI;
use std::fs::File;
//use std::convert::Into;
use std::sync::mpsc;
use wav::{self, bit_depth::BitDepth};
use cpal::{BufferSize, StreamConfig, SampleRate};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

pub struct Resampler {
    n: usize,
    m: usize,
    phasefilt: Vec<Vec<f32>>,
    delayl: Vec<f32>,
    delayr: Vec<f32>,
    delaypos: usize,
    order: usize,
    phase: usize,
}

fn sinc(x: f32) -> f32 {
    if x.abs() < 1e-9 { 1.0 } else { x.sin() / x }
}

impl Resampler {
    // n is upsampling factor, m is downsampling factor.
    // atten is filter band attenuation in dB (around 70).
    // cutoff is a fraction of the original nyquist frequency (like 0.9)
    // order is the number of filter coefficients evaluated per output (like 32)
    fn make_fir(n: usize, atten: f32, cutoff: f32, order: usize) -> Vec<Vec<f32>> {
        // generate FIR coefficients as windowed sinc
        let wc = PI * cutoff / (n as f32);
        let alpha = -325.1e-6 * atten*atten + 0.1677 * atten - 3.149;
        let fullorder = order * n;
        let mid = (fullorder as f32 - 1.0) / 2.0;
        let mut filt: Vec<f32> = (0..fullorder).map(|k| {
                // cosh window: https://dsp.stackexchange.com/questions/37714/kaiser-window-approximation
                let x = k as f32 - mid;
                let normx = 2.0 * x / (fullorder as f32);
                let win = ((1.0 - normx*normx).sqrt() * alpha).cosh() / alpha.cosh();
                sinc(x * wc) * win
            }).collect();

        // distribute coefficients into N phase filters
        let mut phasefilt: Vec<Vec<f32>> = (0..n).map(|_| Vec::new()).collect();
        for (k, coeff) in filt.iter().enumerate() {
            phasefilt[k % n].push(*coeff);
        }
        phasefilt
    }

    pub fn new(n: usize, m: usize, atten: f32, cutoff: f32, order: usize) -> Self {
        let filt = Self::make_fir(n, atten, cutoff, order);
        Resampler {
            n: n,
            m: m,
            phasefilt: filt,
            delayl: vec![0.0; order],
            delayr: vec![0.0; order],
            delaypos: 0,
            order: order,
            phase: 0,
        }
    }

    pub fn resample<F: Fn(f32,f32)>(&mut self, inl: f32, inr: f32, cb: F) {
        // add new sample to delay line
        self.delayl[self.delaypos] = inl;
        self.delayr[self.delaypos] = inr;

        // generate output samples
        while self.phase < self.n {
            let filt = &self.phasefilt[self.phase];
            self.phase += self.m;

            // convolution with selected phase filter
            let mut outl = 0.0;
            let mut outr = 0.0;
            let mut pos = self.delaypos;
            for coef in filt.iter() {
                outl += self.delayl[pos] * coef;
                outr += self.delayr[pos] * coef;
                pos += 1;
                if pos == self.order {
                    pos = 0;
                }
            }
            cb(outl, outr);
        }

        self.phase -= self.n;
        self.delaypos = if self.delaypos == 0 {
                self.order - 1
            } else {
                self.delaypos - 1
            };
    }
}

pub struct Speaker {
    resampler: Resampler,
    tx: mpsc::SyncSender<(f32, f32)>,

    #[allow(dead_code)]
    stream: cpal::Stream, // held until speaker is discarded
}

#[allow(dead_code)]
impl Speaker {
    pub fn new() -> Self {
        // resample 44100->48000 (160/147 ratio), LP 70dB down with cutoff 80% of 22050 (17640Hz), 32-order FIR
        println!("gen filter");
        let resampler = Resampler::new(160, 147, 70.0, 0.8, 32);
        println!("setup au");

        let host = cpal::default_host();
        let dev = host.default_output_device().expect("cant get audio device");
        let cfg = StreamConfig{
            channels: 2,
            sample_rate: SampleRate(48000),
            buffer_size: BufferSize::Default,
        };
        let (tx, rx) = mpsc::sync_channel(64); // XXX parameter
        
        let pump_func = move |dat: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for n in (0..dat.len()).step_by(2) {
                    let (r,l) = rx.recv().unwrap_or((0.0, 0.0));
                    dat[n] = r;
                    dat[n+1] = l;
                }
            };
        let err_func = |err| { eprintln!("audio output error: {}", err); };
        let stream = dev.build_output_stream(&cfg, pump_func, err_func).expect("cant open audio");
        stream.play().unwrap();

        Speaker{ resampler: resampler, tx: tx, stream: stream }
    }

    pub fn play(&mut self, inl: f32, inr: f32) {
        let tx = &self.tx;
        self.resampler.resample(inl, inr, |outl, outr| tx.send((outl, outr)).unwrap());
    }
}

fn main() {
    let mut au = Speaker::new();

    let path = "test.wav";
    let mut inp = File::open(path).expect("couldnt open file");
    println!("read au");
    let (_hdr, dat) = wav::read(&mut inp).expect("couldn't read samples");
    if let BitDepth::Sixteen(vs) = dat {
        println!("play au");
        for i in (0..vs.len()).step_by(2) {
            let left = (vs[i] as f32) / 32768.0;
            let right = (vs[i+1] as f32) / 32768.0;
            au.play(left, right);
        }
    } else {
        panic!("wrong format");
    }
}
