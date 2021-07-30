use std::f64::consts::PI;
use num_complex::Complex;
use eframe::{egui, epi};
use egui::{remap, Color32};
use egui::widgets::plot::{Line, Values, Value, Plot};

const FSAMP: f64 = 44100.0;
const FNYQ: f64 = FSAMP / 2.0;
const MINDB: f64 = -30.0;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Mode { LP, LowShelf, BP, HighShelf, HP }

struct Filt {
    a1: f64,
    a2: f64,
    b0: f64,
    b1: f64,
    b2: f64,
}

fn to_radians(freq: f64) -> f64 {
    freq * PI / FSAMP
}

fn to_db(pow: f64) -> f64 {
    let db = 10.0 * pow.log(10.0);
    if db.is_nan() || db < MINDB { MINDB } else { db }
}

impl Filt {
    fn from_app(app: &App) -> Self {
        Self::from_params(app.freq, app.q, app.gain, app.mode)
    }

    // reference: https://webaudio.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html
    fn from_params(freq: f64, q: f64, gain: f64, mode: Mode) -> Self {
        #[allow(non_snake_case)]
        let A = 10.0_f64.powf(gain/40.0);
        let w = 2.0 * PI * freq / FSAMP;
        let cw = w.cos();
        let sw = w.sin();
        let alpha = 0.5 * sw / q;
        match mode {
        Mode::LowShelf => {
            //let beta = A.sqrt();
            let g = A.sqrt() * sw / q; // short name for: 2 sqrt(A) alpha
            let a0 = (A+1.0) + (A-1.0) * cw + g;
            Filt {
                b0: A * ((A+1.0) - (A-1.0) * cw + g) / a0,
                b1: 2.0 * A * ((A-1.0) - (A+1.0) * cw) / a0,
                b2: A * ((A+1.0) - (A-1.0) * cw - g) / a0,
                a1: -2.0 * ((A-1.0) + (A+1.0) * cw) / a0,
                a2: ((A+1.0) + (A-1.0) * cw - g) / a0,
            }
        },
        Mode::HighShelf => {
            let g = A.sqrt() * sw / q; // short name for: 2 sqrt(A) alpha
            let a0 = (A+1.0) - (A-1.0) * cw + g;
            Filt {
                b0: A * ((A+1.0) + (A-1.0) * cw + g) / a0,
                b1: -2.0 * A * ((A-1.0) + (A+1.0) * cw) / a0,
                b2: A * ((A+1.0) + (A-1.0) * cw - g) / a0,
                a1: 2.0 * ((A-1.0) - (A+1.0) * cw) / a0,
                a2: ((A+1.0) - (A-1.0) * cw - g) / a0,
            }
        },
        Mode::LP => {
            let a0 = 1.0 + alpha / A;
            Filt {
                b0: 0.5 * (1.0 - cw) / a0,
                b1: (1.0 - cw) / a0,
                b2: 0.5 * (1.0 - cw) / a0,
                a1: -2.0 * cw / a0,
                a2: (1.0 - alpha) / a0,
            }
        },
        Mode::HP => {
            let a0 = 1.0 + alpha / A;
            Filt {
                b0: 0.5 * (1.0 + cw) / a0,
                b1: -1.0 * (1.0 + cw) / a0,
                b2: 0.5 * (1.0 + cw) / a0,
                a1: -2.0 * cw / a0,
                a2: (1.0 - alpha) / a0,
            }
        },
        Mode::BP => {
            let a0 = 1.0 + alpha / A;
            Filt {
                b0: (1.0 + alpha * A) / a0,
                b1: (-2.0 * cw) / a0,
                b2: (1.0 - alpha * A) / a0,
                a1: (-2.0 * cw) / a0,
                a2: (1.0 - alpha / A) / a0,
            }
        }
        }
    }

    // frequency response at radian frequency w
    fn response(&self, w: f64) -> f64 {
        let z = Complex::new(w.cos(), w.sin());
        let z2 = z*z;
        let r = (self.b2 + self.b1 * z + self.b0 * z2) / (self.a2 + self.a1 * z + z2);
        r.norm_sqr() // square power
    }
}

struct App {
    freq: f64,
    q: f64,
    gain: f64,
    mode: Mode,
}

impl Default for App {
    fn default() -> Self {
        App { 
            freq: 440.0, 
            q: 1.0,
            gain: 1.0,
            mode: Mode::LowShelf,
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str { "Filter Fun" }
    //fn setup(&mut self, _ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>, _storage: Option<&dyn epi::Storage>) { }
    //fn save(&mut self, _storage: &mut dyn epi::Storage) { }
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let curve = self.freq_curve();
        let Self { freq, q, gain, mode } = self;
        egui::TopBottomPanel::top("Filter Fun").show(ctx, |ui| {
            ui.heading("Controls");
            ui.add(egui::Slider::new(freq, 0.0..=FNYQ).text("Freq"));
            ui.add(egui::Slider::new(q, 0.1..=10.0).text("Q"));
            ui.add(egui::Slider::new(gain, -5.0..=5.0).text("Gain"));
            ui.horizontal(|ui| {
                ui.radio_value(mode, Mode::LowShelf, "LowShelf");
                ui.radio_value(mode, Mode::LP, "LP");
                ui.radio_value(mode, Mode::BP, "BP");
                ui.radio_value(mode, Mode::HP, "HP");
                ui.radio_value(mode, Mode::HighShelf, "HighShelf");
            });
            let plot = Plot::new("freq response")
                .line(curve)
                .view_aspect(1.5)
                .include_y(MINDB)
                .include_y(2.0)
                ;
            ui.add(plot);
        });
    }
}

impl App {
    fn freq_curve(&self) -> Line {
        let filt = Filt::from_app(self);
        let n = 512;
        let dat = (0..n).map(|i| {
            let freq = remap(i as f64, 0.0..=(n as f64), 0.0..=FNYQ);
            let gain = to_db(filt.response(to_radians(freq)));
            //let gain = filt.response(to_radians(freq));
            Value::new(freq, gain)
        });
        Line::new(Values::from_values_iter(dat))
            .color(Color32::from_rgb(100, 200, 100))
            .name("FreqResponse")
    }
}

fn main() {
    let app = App::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
