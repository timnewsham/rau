use std::f64::consts::PI;
use eframe::{egui, epi};
use egui::{Color32, remap};
use egui::widgets::plot::{Line, Values, Value, Plot};

#[derive(PartialEq, Copy, Clone)]
enum Function { SIN, TRI, SAW, SQUARE }

struct App {
    func: Function,
    freq: f64,
    n: usize,
}

impl App {
    fn new() -> Self {
        App { func: Function::TRI, freq: 3.0, n: 3 }
    }
}

// shorthand
fn powneg1(k: usize) -> f64 {
    (-1.0_f64).powf(k as f64)
}

fn make_series(func: Function, n: usize) -> Vec<(usize, f64)> {
    match func {
        Function::SIN => vec![(1, 1.0)],
        Function::SAW => (1..=n).map(|k|
                (k, -2.0 * powneg1(k) / (k as f64 * PI))
            ).collect(),
        Function::TRI => (1..=n).map(|nn| {
                let k = 2*nn - 1; // odd harmonics
                (k, 8.0 * powneg1((k-1)/2) / (k as f64 * PI).powf(2.0))
            }).collect(),
        Function::SQUARE => (1..=n).map(|nn| {
                let k = 2*nn - 1; // odd harmonics
                (k, -4.0 * powneg1(k) / (k as f64 * PI))
            }).collect(),
    }
}

// evaluate the fourier series at time t for freq w (in radians)
fn calc_series(t: f64, w: f64, series: &Vec<(usize, f64)>) -> f64 {
    series.iter().map(|(k, b)| b * (w*t*(*k as f64)).sin()).sum()
}

fn curve(freq: f64, series: &Vec<(usize, f64)>) -> Line {
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

impl epi::App for App {
    fn name(&self) -> &str { "Fourier Fun" }
    //fn setup(&mut self, _ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>, _storage: Option<&dyn epi::Storage>) { }
    //fn save(&mut self, _storage: &mut dyn epi::Storage) { }
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let Self { func, freq, n } = self;
        let series = make_series(*func, *n);
        egui::TopBottomPanel::top("Fourier Fun").show(ctx, |ui| {
            ui.heading("Controls");
            ui.add(egui::Slider::new(freq, 1.0..=10.0).text("Freq"));
            ui.add(egui::Slider::new(n, 1..=20).text("Waves"));
            ui.horizontal(|ui| {
                ui.radio_value(func, Function::SIN, "SIN");
                ui.radio_value(func, Function::TRI, "TRI");
                ui.radio_value(func, Function::SAW, "SAW");
                ui.radio_value(func, Function::SQUARE, "SQUARE");
            });

            let curve = curve(*freq, &series);
            let plot = Plot::new("phase plot")
                .line(curve)
                .view_aspect(1.5)
                .include_y(-1.5)
                .include_y(1.5)
                .include_x(0.0)
                .include_x(1.0)
                ;
            ui.add(plot);
        });
    }
}

fn main() {
    let app = App::new();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
