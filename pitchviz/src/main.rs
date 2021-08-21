
use std::env::args;
//use std::cmp;
use eframe::{egui, epi};
use egui::Color32;
use egui::widgets::plot::{Line, Points, Values, Value, Plot, Legend};
//use rau::speaker::{Sample, MidSide, ResamplingSpeaker};
use rau::wav::{read_wav, Sample};
use rau::pitch::Pitch;
use rau::units::{Cent, Sec, Samples};

const FSAMP: f64 = 48000.0;

struct App {
    //speaker: ResamplingSpeaker,
    pitches: Vec<(Option<Cent>, f64)>,
}

impl App {
    fn new(pitches: Vec<(Option<Cent>, f64)>) -> Self {
        App {
            pitches: pitches,
        }
    }
}

fn curve(pitches: &Vec<(Option<Cent>, f64)>) -> (Points, Line)
{
    let dat1 = pitches.iter().enumerate().filter(|(n,p)| p.0.is_some()).map(|(n,p)| {
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
        .name("Corr");
    (p1, l2)
}


impl epi::App for App {
    fn name(&self) -> &str { "Phase Fun" }
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let Self { pitches } = self;
        egui::TopBottomPanel::top("Filter Fun").show(ctx, |ui| {
            ui.heading("Controls");
            let (curve1, curve2) = curve(&*pitches);
            let plot = Plot::new("pitch plot")
                .points(curve1)
                .line(curve2)
                .view_aspect(1.5)
                .legend(Legend::default())
                ;
            ui.add(plot);
        });
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    let path = if args.len() > 1 { &args[1] } else { "pitch.wav" };

    println!("read file");
    let samples = read_wav(path, FSAMP);

    println!("compute pitches");
    //let mut p = Pitch::new(Sec(0.050), Sec(0.010));
    let mut p = Pitch::new(Sec(0.030), Sec(0.002));
    let mut dat: Vec<(Option<Cent>, f64)> = Vec::new();
    for Sample{left, right: _} in samples {
        if let Some(x) = p.add_sample2(left) {
            dat.push(x);
        }
    }

    println!("gui");
    let app = App::new(dat);
    let mut native_options = eframe::NativeOptions::default();
    native_options.drag_and_drop_support = false;
    eframe::run_native(Box::new(app), native_options);
}
