//use std::time::Duration;
//use std::thread::sleep;
use std::sync::Arc;
use std::cmp;
use std::fs::File;
use std::sync::mpsc;
use num_complex::Complex;
use wav::{self, bit_depth::BitDepth};
use eframe::{egui, epi};
use egui::{Color32, NumExt, remap};
use egui::widgets::plot::{Line, Values, Value, Plot, Legend};
use cpal::{BufferSize, StreamConfig, SampleRate};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use rustfft::*;

//const FSAMP: f64 = 48000.0;
const FSAMP: f64 = 44100.0;
const MAXHZ: f64 = 0.5 * FSAMP;
//const FFTSIZE: usize = 512;
const FFTSIZE: usize = 1024;
const MINDB: f64 = -60.0;

struct Sample {
    left: f64,
    right: f64,
}

// XXX dummy out tx for now because cpal audio triggers a crash in egui right now
// on windows due to some COM stuff that needs to be fixed in egui.
struct App {
    tx: usize, // XXX mpsc::SyncSender<(f32, f32)>,
    samples: Vec<Sample>,
    time: f64,
    fft: Arc<dyn Fft<f64>>,
    midhist: Vec<f64>,
    sidehist: Vec<f64>,
    alpha: f64,
}

#[allow(dead_code)]
fn open_speaker() -> mpsc::SyncSender<(f32, f32)> {
    let host = cpal::default_host();
    let dev = host.default_output_device().expect("cant get audio device");
    let cfg = StreamConfig{
        channels: 2,
        sample_rate: SampleRate(FSAMP as u32),
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
    tx
}

impl App {
    fn from_file(path: &str) -> Self {
        let tx = 0; // = open_speaker();
        let mut inp = File::open(path).expect("couldnt open file");
        let (_hdr, dat) = wav::read(&mut inp).expect("couldn't read samples");
        let mut samples = Vec::new();
        if let BitDepth::Sixteen(vs) = dat {
            for i in (0..vs.len()).step_by(2) {
                let right = (vs[i] as f64) / 32768.0;
                let left = (vs[i+1] as f64) / 32768.0;
                samples.push(Sample{ left: left, right: right } );
            }
        } else {
            panic!("wrong format");
        }

        let mut planner = FftPlanner::new();
        App {
            tx: tx,
            samples: samples,
            time: 0.0,
            fft: planner.plan_fft_forward(FFTSIZE),
            midhist: vec![0.0; FFTSIZE],
            sidehist: vec![0.0; FFTSIZE],
            alpha: 0.5,
        }
    }

    fn max_time(&self) -> f64 {
        self.samples.len() as f64 / FSAMP
    }
}

fn to_db(pow: f64) -> f64 {
    let db = 10.0 * pow.log(10.0);
    if db.is_nan() || db < MINDB { MINDB } else { db }
}

// pack mid and side into a complex number
// if Z is the FFT of this, we can recover separate mid and side FFTs as:
//   midfft[n] = (Z[n] + Z*[N-n])/2
//   sidefft[n] = -j * (Z[n] - Z*[n])/2 
fn mid_side(s: &Sample) -> Complex<f64> {
    let mid = 0.5 * (s.left + s.right);
    let side = 0.5 * (s.left - s.right);
        //Complex{ re: mid, im: 0.0 } // XXX test dual-DFT out
        //Complex{ re: 0.0, im: mid } // XXX test dual-DFT out
    Complex{ re: mid, im: side }
}

fn curve(_tx: &usize /*mpsc::SyncSender<(f32, f32)>*/, 
        fft: &Arc<dyn Fft<f64>>,
        midhist: &mut Vec<f64>,
        sidehist: &mut Vec<f64>,
        alpha: f64,
        samps: &Vec<Sample>, from_t: f64, to_t: f64) -> (Line, Line)
{
    // compute window size for FFTSIZE samples
    let mut from = (from_t * FSAMP) as usize;
    let mut to = cmp::min((to_t * FSAMP) as usize, samps.len());
    if to >= FFTSIZE {
        from = to - (FFTSIZE - 1);
    } else {
        to = from + (FFTSIZE - 1);
    }
    assert!(to - from == FFTSIZE - 1 && from < samps.len() && to < samps.len());

    // gather window and fft
    // note: we're doing two FFT's here because real=mid and imaj=side
    // note: we're most likely dropping info here.. but should be "good enough"
    // we usually get chunks of 800 samples or so at a time.. so we're missing 300 each time..
    let mut v: Vec<Complex<f64>> = (0..FFTSIZE).map(|n| mid_side(&samps[from + n])).collect();
    fft.process(&mut v);
    let scale = 1.0 / (FFTSIZE as f64).sqrt();
    v.iter_mut().for_each(|x| *x *= scale);

    // recover our two FFTs and
    // mix new values into our history for smoothing...
    for n in 1..(FFTSIZE/2) {
        let mid = (v[n] + v[FFTSIZE-n].conj()) / 2.0;
        let side = - Complex::<f64>::i() * (v[n] - v[FFTSIZE-n].conj()) / 2.0;
        let mid_db = to_db(mid.norm_sqr());
        let side_db = to_db(side.norm_sqr());
        midhist[n] = mid_db * alpha + midhist[n] * (1.0 - alpha);
        sidehist[n] = side_db * alpha + sidehist[n] * (1.0 - alpha);
    }

    let dat1 = (3..FFTSIZE/2).map(|i| {
            let freq = remap(i as f64, 0.0..=((FFTSIZE/2) as f64), 0.0..=MAXHZ);
            Value::new(freq.log(10.0), midhist[i])
        });
    let l1 = Line::new(Values::from_values_iter(dat1))
        .color(Color32::from_rgb(100, 200, 100))
        .name("MID FFT");

    let dat2 = (3..FFTSIZE/2).map(|i| {
            let freq = remap(i as f64, 0.0..=((FFTSIZE/2) as f64), 0.0..=MAXHZ);
            Value::new(freq.log(10.0), sidehist[i])
        });
    let l2 = Line::new(Values::from_values_iter(dat2))
        .color(Color32::from_rgb(100, 100, 200))
        .name("SIDE FFT");
    return (l1, l2);
}


impl epi::App for App {
    fn name(&self) -> &str { "Phase Fun" }
    //fn setup(&mut self, _ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>, _storage: Option<&dyn epi::Storage>) { }
    //fn save(&mut self, _storage: &mut dyn epi::Storage) { }
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let maxt = self.max_time();
        let Self { tx, samples, time, fft, midhist, sidehist, alpha } = self;
        egui::TopBottomPanel::top("Filter Fun").show(ctx, |ui| {
            ui.ctx().request_repaint(); // always repaint, it advances our clock

            let starttime = *time;
            let endtime = *time + ui.input().unstable_dt.at_most(1.0 / 30.0) as f64;
            if endtime < maxt {
                *time = endtime;
            } else {
                *time = 0.0;
            }
            //ui.heading(format!("Controls - time {:.01}   samples {:.0}", starttime, (endtime-starttime)*FSAMP));
            ui.heading("Controls");
            ui.add(egui::Slider::new(alpha, 0.01..=0.99).text("Alpha"));
            ui.add(egui::Slider::new(time, 0.0..=maxt).text("Time"));

            let (curve1, curve2) = curve(&*tx, &*fft, &mut *midhist, &mut *sidehist, *alpha, samples, starttime, endtime);
            let plot = Plot::new("phase plot")
                .line(curve1)
                .line(curve2)
                .view_aspect(1.5)
                .include_y(15.0)
                .include_y(MINDB)
                //.include_x(-10.0)
                .include_x(MAXHZ.log(10.0))
                .legend(Legend::default())
                ;
            ui.add(plot);
        });
    }
}

fn main() {
    let app = App::from_file("test.wav");
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
