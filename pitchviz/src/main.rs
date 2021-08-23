
use std::env::args;
use eframe::{egui, epi};
use egui::Color32;
use egui::widgets::plot::{Line, Points, Values, Value, Plot, Legend};
//use rau::speaker::{Sample, MidSide, ResamplingSpeaker};
use rau::wav::{read_wav_at, Sample};
use rau::pitch::{Pitch, period_to_note};
use rau::units::{Cent, Sec, Samples};

#[derive(PartialEq, Clone, Copy, Debug)]
enum View { Pitch, NSDFDelay, NSDFPitch }

const FSAMP: f64 = 48000.0;

struct App {
    //speaker: ResamplingSpeaker,
    pitches: Vec<(Option<Cent>, f64)>,
    nsdfs: Vec<Vec<f64>>,
    view: View,
    time: usize,
}

impl App {
    fn new(pitches: Vec<(Option<Cent>, f64)>, nsdfs: Vec<Vec<f64>>) -> Self {
        App {
            pitches, nsdfs,
            view: View::Pitch,
            time: 0,
        }
    }
}

fn nsdf_curve(nsdf: &Vec<f64>, show_delay: bool) -> Line {
    let dat = nsdf.iter().enumerate().filter(|(n,_)| *n > 0).map(|(n,r)| {
            let samps = Samples(n);
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
        .name("NSDF")
}

fn pitch_curve(pitches: &Vec<(Option<Cent>, f64)>, time: f64) -> (Points, Line, Points)
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
            Value::new(sec, 1000.0 * p.1)
        });
    let l2 = Line::new(Values::from_values_iter(dat2))
        .color(Color32::from_rgb(200, 100, 100))
        .name("Clarity");

    let Sec(now) = Sec(time).into();
    let dat_n = vec![Value::new(now, 0.0)];
    let p_n = Points::new(Values::from_values(dat_n))
        .radius(5.0)
        .color(Color32::from_rgb(100, 100, 200))
        .name("Time");

    (p1, l2, p_n)
}


impl epi::App for App {
    fn name(&self) -> &str { "Phase Fun" }
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let Self { pitches, nsdfs, view, time } = self;
        let Sec(now) = Samples(*time).into();
        egui::TopBottomPanel::top("Filter Fun").show(ctx, |ui| {
            ui.heading("Controls");
            ui.add(egui::Slider::new(time, 0..=nsdfs.len()-1).text(format!("Time {}", now)));
            ui.horizontal(|ui| {
                ui.radio_value(view, View::Pitch, "Pitch");
                ui.radio_value(view, View::NSDFPitch, "NSDFPitch");
                ui.radio_value(view, View::NSDFDelay, "NSDFDelay");
            });

            match view {
                View::Pitch => {
                    let (curve1, curve2, curve3) = pitch_curve(&*pitches, now);
                    let plot = Plot::new("pitch plot")
                        .points(curve1)
                        .line(curve2)
                        .points(curve3)
                        .view_aspect(1.5)
                        .legend(Legend::default())
                        ;
                    ui.add(plot);
                },
                View::NSDFPitch 
                | View::NSDFDelay => {
                    let curve = nsdf_curve(&nsdfs[*time], *view == View::NSDFDelay);
                    let plot = Plot::new("nsdf plot")
                        .line(curve)
                        .view_aspect(1.5)
                        .include_y(1.5)
                        .include_y(-1.5)
                        .legend(Legend::default())
                        ;
                    ui.add(plot);
                },
            }
        });
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    let path = if args.len() > 1 { &args[1] } else { "pitch.wav" };

    println!("read file");
    let samples = read_wav_at(path, FSAMP);

    println!("compute pitches and NSDFs");
    //let mut p = Pitch::new(Sec(0.050), Sec(0.010));
    let mut p = Pitch::new(Sec(0.030), Sec(0.002));
    let mut nsdfs: Vec<Vec<f64>> = Vec::new();
    let mut pitches: Vec<(Option<Cent>, f64)> = Vec::new();
    for Sample{left, right: _} in samples {
        if let Some(_) = p.proc_sample(left) {
            nsdfs.push(p.nsdf.buf[0 .. p.nsdf.k].iter().copied().collect());
            pitches.push((p.note, p.clarity));
        }
    }

    println!("gui");
    let app = App::new(pitches, nsdfs);
    let mut native_options = eframe::NativeOptions::default();
    native_options.drag_and_drop_support = false;
    eframe::run_native(Box::new(app), native_options);
}
