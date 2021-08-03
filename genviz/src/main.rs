use std::f64::consts::PI;
use eframe::{egui, epi};
use egui::{Color32, remap};
use egui::widgets::plot::{Line, Values, Value, Plot};

use rau::additive::{Function, Gen, HarmonicParam};
use rau::units::{Hz};

struct App {
    func: Function,
    freq: f64,
    n: usize,
    gen: Gen,
}

impl App {
    fn new() -> Self {
        App { 
            func: Function::TRI, freq: 3.0, n: 3, 
            gen: Gen::new(Function::TRI, Hz(3.0), 3)
        }
    }
}

// evaluate the fourier series at time t for freq w (in radians)
fn calc_series(t: f64, w: f64, series: &Vec<HarmonicParam>) -> f64 {
    series.iter().map(|HarmonicParam{k, amp}| *amp * (w*t*(*k as f64)).sin()).sum()
}

fn curve(freq: f64, series: &Vec<HarmonicParam>) -> Line {
    let wfreq = freq * 2.0 * PI;
    let n = 512;
    let dat = (0..n).map(|i| {
            let t = remap(i as f64, 0.0..=(n as f64), 0.0..=1.0);
            Value::new(t, calc_series(t, wfreq, series))
        });
    Line::new(Values::from_values_iter(dat))
        .color(Color32::from_rgb(100, 200, 100))
        .name("Waveform")
}

fn harmonics(series: &Vec<HarmonicParam>) -> Vec<Line> {
    series.iter().map(|HarmonicParam{k, amp}| {
            let x = *k as f64 / 30.0;
            let vs = vec!(Value::new(x, 0.0), Value::new(x, *amp));
            Line::new(Values::from_values(vs))
                .color(Color32::from_rgb(250,050,050))
                .name("weight")
        }).collect()
}

impl epi::App for App {
    fn name(&self) -> &str { "Fourier Fun" }
    //fn setup(&mut self, _ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>, _storage: Option<&dyn epi::Storage>) { }
    //fn save(&mut self, _storage: &mut dyn epi::Storage) { }
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let Self { func, freq, n, gen } = self;
        gen.set_func(*func, *n);
        let series = &gen.series;
        egui::TopBottomPanel::top("Fourier Fun").show(ctx, |ui| {
            ui.heading("Controls");
            ui.add(egui::Slider::new(freq, 1.0..=10.0).text("Freq"));
            ui.add(egui::Slider::new(n, 1..=20).text("Waves"));
            ui.horizontal(|ui| {
                ui.radio_value(func, Function::SIN, "SIN");
                ui.radio_value(func, Function::TRI, "TRI");
                ui.radio_value(func, Function::SAWUP, "SAWUP");
                ui.radio_value(func, Function::SAWDOWN, "SAWDOWN");
                ui.radio_value(func, Function::SQUARE, "SQUARE");
            });

            let curve = curve(*freq, &series);
            let mut plot = Plot::new("phase plot")
                .line(curve)
                .view_aspect(1.5)
                .include_y(-1.5)
                .include_y(1.5)
                .include_x(0.0)
                .include_x(1.0)
                ;
            for l in harmonics(&series) {
                plot = plot.line(l);
            }
            ui.add(plot);
        });
    }
}

fn main() {
    let app = App::new();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
