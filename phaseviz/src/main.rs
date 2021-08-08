
use std::env::args;
use std::cmp;
use eframe::{egui, epi};
use egui::{Color32, NumExt};
use egui::widgets::plot::{Line, Values, Value, Plot};
use rau::speaker::{Sample, MidSide, ResamplingSpeaker};
use rau::wav::read_wav;

const FSAMP: f64 = 44100.0;

struct App {
    speaker: ResamplingSpeaker,
    samples: Vec<Sample>,
    time: f64,
}

impl App {
    fn from_file(path: &str) -> Self {
        let speaker = ResamplingSpeaker::new_441_to_480(1000);
        let samples = read_wav(path, FSAMP);

        App {
            speaker: speaker,
            samples: samples,
            time: 0.0,
        }
    }

    fn max_time(&self) -> f64 {
        self.samples.len() as f64 / FSAMP
    }
}

fn phase_curve(speaker: &mut ResamplingSpeaker, samps: &Vec<Sample>, from_t: f64, to_t: f64) -> Line {
    let from = (from_t * FSAMP) as usize;
    let to = cmp::min((to_t * FSAMP) as usize, samps.len());
    let dat = (from..to).map(|i| {
            speaker.play(samps[i]);
            let MidSide{ mid, side } = samps[i].into();
            Value::new(side, mid)
        });
        // XXX lines for now, but perhaps better with points?
        Line::new(Values::from_values_iter(dat))
            .color(Color32::from_rgb(100, 200, 100))
            .name("FreqResponse")
    }


impl epi::App for App {
    fn name(&self) -> &str { "Phase Fun" }
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let maxt = self.max_time();
        let Self { speaker, samples, time } = self;
        egui::TopBottomPanel::top("Filter Fun").show(ctx, |ui| {
            ui.ctx().request_repaint(); // always repaint, it advances our clock

            let starttime = *time;
            let endtime = *time + ui.input().unstable_dt.at_most(1.0 / 30.0) as f64;
            if endtime < maxt {
                *time = endtime;
            } else {
                *time = 0.0;
            }
            ui.heading("Controls");
            ui.add(egui::Slider::new(time, 0.0..=maxt).text("Time"));

            let curve = phase_curve(speaker, samples, starttime, endtime);
            let plot = Plot::new("phase plot")
                .line(curve)
                .view_aspect(1.5)
                .include_y(-1.0)
                .include_y(1.0)
                .include_x(-1.0)
                .include_x(1.0)
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
