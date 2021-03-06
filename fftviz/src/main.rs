
use std::env::args;
use std::sync::Arc;
use std::cmp;
use num_complex::Complex;
use eframe::{egui, epi};
use egui::{Color32, NumExt, remap};
use egui::widgets::plot::{Line, Values, Value, Plot, Legend};
use rustfft::*;
use rau::speaker::{Sample, MidSide, player_at, DynSpeaker};
use rau::wav::read_wav;

const FFTSIZE: usize = 1024;
const MINDB: f64 = -60.0;

struct App {
    speaker: DynSpeaker,
    samples: Vec<Sample>,
    time: f64,
    fft: Arc<dyn Fft<f64>>,
    midhist: Vec<f64>,
    sidehist: Vec<f64>,
    alpha: f64,
    fsamp: f64,
}

impl App {
    fn from_file(path: &str) -> Self {
        let (fsamp, samples) = read_wav(path);
        let speaker = player_at(fsamp, 1000);
        let mut planner = FftPlanner::new();

        App {
            speaker: speaker,
            samples: samples,
            time: 0.0,
            fft: planner.plan_fft_forward(FFTSIZE),
            midhist: vec![0.0; FFTSIZE],
            sidehist: vec![0.0; FFTSIZE],
            alpha: 0.5,
            fsamp: fsamp as f64,
        }
    }

    fn max_time(&self) -> f64 {
        self.samples.len() as f64 / self.fsamp
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
    let MidSide{ mid, side } = (*s).into();
        //Complex{ re: mid, im: 0.0 } // XXX test dual-DFT out
        //Complex{ re: 0.0, im: mid } // XXX test dual-DFT out
    Complex{ re: mid, im: side }
}

fn curve(speaker: &mut DynSpeaker,
        fft: &Arc<dyn Fft<f64>>,
        midhist: &mut Vec<f64>,
        sidehist: &mut Vec<f64>,
        alpha: f64,
        samps: &Vec<Sample>, from_t: f64, to_t: f64,
        fsamp: f64) -> (Line, Line)
{
    // compute window extents for FFTSIZE samples
    // and deliver the audio
    let mut from = (from_t * fsamp) as usize;
    let mut to = cmp::min((to_t * fsamp) as usize, samps.len());
    for i in from..to {
        speaker.play(samps[i]);
    }

    if to >= FFTSIZE {
        from = to - FFTSIZE;
    } else {
        to = from + FFTSIZE;
    }
    assert!(to - from == FFTSIZE && from < samps.len() && to <= samps.len());

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

    let maxhz = 0.5 * fsamp;
    let dat1 = (3..FFTSIZE/2).map(|i| {
            let freq = remap(i as f64, 0.0..=((FFTSIZE/2) as f64), 0.0..=maxhz);
            Value::new(freq.log(10.0), midhist[i])
        });
    let l1 = Line::new(Values::from_values_iter(dat1))
        .color(Color32::from_rgb(100, 200, 100))
        .name("MID FFT");

    let dat2 = (3..FFTSIZE/2).map(|i| {
            let freq = remap(i as f64, 0.0..=((FFTSIZE/2) as f64), 0.0..=maxhz);
            Value::new(freq.log(10.0), sidehist[i])
        });
    let l2 = Line::new(Values::from_values_iter(dat2))
        .color(Color32::from_rgb(100, 100, 200))
        .name("SIDE FFT");
    return (l1, l2);
}


impl epi::App for App {
    fn name(&self) -> &str { "Phase Fun" }
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let maxt = self.max_time();
        let Self { speaker, samples, time, fft, midhist, sidehist, alpha, fsamp, .. } = self;
        let maxhz = 0.5 * *fsamp;
        egui::TopBottomPanel::top("Filter Fun").show(ctx, |ui| {
            ui.ctx().request_repaint(); // always repaint, it advances our clock

            let starttime = *time;
            let endtime = *time + ui.input().unstable_dt.at_most(1.0 / 30.0) as f64;
            if endtime < maxt {
                *time = endtime;
            } else {
                *time = 0.0;
            }
            //ui.heading(format!("Controls - time {:.01}   samples {:.0}", starttime, (endtime-starttime)*fsamp));
            ui.heading("Controls");
            ui.add(egui::Slider::new(alpha, 0.01..=0.99).text("Alpha"));
            ui.add(egui::Slider::new(time, 0.0..=maxt).text("Time"));

            let (curve1, curve2) = curve(&mut *speaker, &*fft, &mut *midhist, &mut *sidehist, *alpha, samples, starttime, endtime, *fsamp);
            let plot = Plot::new("phase plot")
                .line(curve1)
                .line(curve2)
                .view_aspect(1.5)
                .include_y(15.0)
                .include_y(MINDB)
                //.include_x(-10.0)
                .include_x(maxhz.log(10.0))
                .legend(Legend::default())
                ;
            ui.add(plot);
        });
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    let path = if args.len() > 1 { &args[1] } else { "test.wav" };

    let app = App::from_file(path);
    let mut native_options = eframe::NativeOptions::default();
    native_options.drag_and_drop_support = false;
    eframe::run_native(Box::new(app), native_options);
}
