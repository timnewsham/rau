
use std::env::args;
use num_complex::Complex;
use eframe::{egui, epi};
use egui::{Color32, remap};
use egui::widgets::plot::{Line, Points, Values, Value, Plot, Legend};
//use rau::speaker::{Sample, MidSide, ResamplingSpeaker};
use rau::wav::{read_wav_at, Sample};
use rau::pitch::{Pitch, period_to_note};
use rau::units::{Cent, Sec, Samples};

#[derive(PartialEq, Clone, Copy, Debug)]
enum View { Pitch, CorrDelay, CorrPitch, CorrFft }

const FSAMP: f64 = 48000.0;
const MINDB: f64 = -60.0;

struct App {
    //speaker: ResamplingSpeaker,
    pitches: Vec<(Option<Cent>, f64, f64)>,
    corrs: Vec<Vec<f64>>,
    ffts: Vec<Vec<Complex<f64>>>,
    mindelay: usize,
    view: View,
    time: usize,
}

impl App {
    fn new(pitches: Vec<(Option<Cent>, f64, f64)>, corrs: Vec<Vec<f64>>, ffts: Vec<Vec<Complex<f64>>>, mindelay: usize) -> Self {
        App {
            pitches, corrs, ffts, mindelay,
            view: View::Pitch,
            time: 0,
        }
    }
}

fn to_db(pow: f64) -> f64 {
    let db = 10.0 * pow.log(10.0);
    if db.is_nan() || db < MINDB { MINDB } else { db }
}

fn corr_curve(corr: &Vec<f64>, mindelay: usize, show_delay: bool) -> Line {
    let dat = corr.iter().enumerate().filter(|(n,_)| *n > 0).map(|(n,r)| {
            let samps = Samples(n + mindelay);
            let x = if show_delay {
                    let Sec(sec) = samps.into();
                    sec
                } else {
                    let Cent(cent) = period_to_note(samps);
                    cent
                };
            Value::new(x, *r)
        });
    Line::new(Values::from_values_iter(dat))
        .color(Color32::from_rgb(100, 200, 100))
        .name("Corr")
}

fn corr_fft_curve(fft: &Vec<Complex<f64>>) -> Line {
    let fftsize = fft.len() / 2;
    let maxhz = FSAMP / 2.0;
    let dat = (3..fftsize).map(|i| {
            let freq = remap(i as f64, 0.0..=(fftsize as f64), 0.0..=maxhz);
            Value::new(freq.log(10.0), to_db(fft[i].norm_sqr()))
        });

    Line::new(Values::from_values_iter(dat))
        .color(Color32::from_rgb(100, 200, 100))
        .name("CorrFft")
}

fn pitch_curve(pitches: &Vec<(Option<Cent>, f64, f64)>, time: f64) -> (Points, Line, Line, Points)
{
    let dat1 = pitches.iter().enumerate().filter(|(_,p)| p.0.is_some()).map(|(n,p)| {
            let Cent(cent) = p.0.unwrap();
            let Sec(sec) = Samples(n).into();
            Value::new(sec, cent)
        });
    let p1 = Points::new(Values::from_values_iter(dat1))
        .color(Color32::from_rgb(100, 200, 100))
        .name("Cents");

    let dat2 = pitches.iter().enumerate().map(|(n,p)| {
            let Sec(sec) = Samples(n).into();
            Value::new(sec, 1000.0 * p.2)
        });
    let l2 = Line::new(Values::from_values_iter(dat2))
        .color(Color32::from_rgb(200, 100, 100))
        .name("Corr");

    let dat3 = pitches.iter().enumerate().map(|(n,p)| {
            let Sec(sec) = Samples(n).into();
            Value::new(sec, 100.0 * p.1)
        });
    let l3 = Line::new(Values::from_values_iter(dat3))
        .color(Color32::from_rgb(200, 200, 100))
        .name("Pow");

    let Sec(now) = Sec(time).into();
    let datN = vec![Value::new(now, 0.0)];
    let pN = Points::new(Values::from_values(datN))
        .radius(5.0)
        .color(Color32::from_rgb(100, 100, 200))
        .name("Time");

    (p1, l2, l3, pN)
}


impl epi::App for App {
    fn name(&self) -> &str { "Phase Fun" }
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let Self { pitches, corrs, ffts, mindelay, view, time } = self;
        let Sec(now) = Samples(*time).into();
        egui::TopBottomPanel::top("Filter Fun").show(ctx, |ui| {
            ui.heading("Controls");
            ui.add(egui::Slider::new(time, 0..=corrs.len()-1).text(format!("Time {}", now)));
            ui.horizontal(|ui| {
                ui.radio_value(view, View::Pitch, "Pitch");
                ui.radio_value(view, View::CorrPitch, "CorrPitch");
                ui.radio_value(view, View::CorrDelay, "CorrDelay");
                ui.radio_value(view, View::CorrFft, "CorrFft");
            });

            match view {
                View::Pitch => {
                    let (curve1, curve2, curve3, curve4) = pitch_curve(&*pitches, now);
                    let plot = Plot::new("pitch plot")
                        .points(curve1)
                        .line(curve2)
                        .line(curve3)
                        .points(curve4)
                        .view_aspect(1.5)
                        .legend(Legend::default())
                        ;
                    ui.add(plot);
                },
                View::CorrPitch 
                | View::CorrDelay => {
                    let curve = corr_curve(&corrs[*time], *mindelay, *view == View::CorrDelay);
                    let plot = Plot::new("corr plot")
                        .line(curve)
                        .view_aspect(1.5)
                        .include_y(1.5)
                        .include_y(-1.5)
                        .legend(Legend::default())
                        ;
                    ui.add(plot);
                },
                View::CorrFft => {
                    let curve = corr_fft_curve(&ffts[*time]);
                    let plot = Plot::new("corrfft plot")
                        .line(curve)
                        .view_aspect(1.5)
                        .include_y(30.0)
                        .include_y(MINDB)
                        .legend(Legend::default())
                        ;
                    ui.add(plot);
                }
            }
        });
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    let path = if args.len() > 1 { &args[1] } else { "pitch.wav" };

    println!("read file");
    let samples = read_wav_at(path, FSAMP);

    println!("compute pitches and autocorrs");
    //let mut p = Pitch::new(Sec(0.050), Sec(0.010));
    let mut p = Pitch::new(Sec(0.030), Sec(0.002));
    let mut corrs: Vec<Vec<f64>> = Vec::new();
    let mut pitches: Vec<(Option<Cent>, f64, f64)> = Vec::new();
    let mut ffts = Vec::new();
    for Sample{left, right: _} in samples {
        if let Some(_) = p.proc_sample(left) {
            corrs.push(p.corrdata.buf[0..p.corrdata.k].iter().map(|v| v.re).collect());
            ffts.push(p.fftdata.iter().copied().collect());
            pitches.push((p.note, to_db(p.power), p.corr));
        }
    }

    println!("gui");
    let app = App::new(pitches, corrs, ffts, p.minscan.0);
    let mut native_options = eframe::NativeOptions::default();
    native_options.drag_and_drop_support = false;
    eframe::run_native(Box::new(app), native_options);
}
