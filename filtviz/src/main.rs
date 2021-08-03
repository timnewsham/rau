use num_complex::Complex;
use eframe::{egui, epi};
use egui::{remap, Color32};
use egui::widgets::plot::{Line, Values, Value, Plot};

use rau::filt::{Filter, FiltType};
use rau::units::{Hz, RadPS, MAXHZ};

const MINDB: f64 = -30.0;

fn to_db(pow: f64) -> f64 {
    let db = 10.0 * pow.log(10.0);
    if db.is_nan() || db < MINDB { MINDB } else { db }
}

struct App {
    freq: f64,
    q: f64,
    gain: f64,
    mode: FiltType,
    filter: Filter,
}

impl App {
    // frequency response at radian frequency w
    fn response(&self, w: f64) -> f64 {
        let filt = &self.filter;
        let z = Complex::new(w.cos(), w.sin());
        let z2 = z*z;
        let r = (filt.b2 + filt.b1 * z + filt.b0 * z2) / (filt.a2 + filt.a1 * z + z2);
        r.norm_sqr() // square power
    }

    fn freq_curve(&self) -> Line {
        let n = 512;
        let dat = (0..n).map(|i| {
            let freq = remap(i as f64, 0.0..=(n as f64), 0.0..=MAXHZ);
            let RadPS(w) = Hz(freq).into();
            let gain = to_db(self.response(w));
            Value::new(freq, gain)
        });
        Line::new(Values::from_values_iter(dat))
            .color(Color32::from_rgb(100, 200, 100))
            .name("FreqResponse")
    }
}

impl Default for App {
    fn default() -> Self {
        App { 
            freq: 440.0, 
            q: 1.0,
            gain: 1.0,
            mode: FiltType::LowShelf,
            filter: Filter::default(),
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str { "Filter Fun" }
    //fn setup(&mut self, _ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>, _storage: Option<&dyn epi::Storage>) { }
    //fn save(&mut self, _storage: &mut dyn epi::Storage) { }
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let curve = self.freq_curve();
        let Self { freq, q, gain, mode, filter } = self;
        *filter = Filter::new(*mode, Hz(*freq), *gain, *q);
        egui::TopBottomPanel::top("Filter Fun").show(ctx, |ui| {
            ui.heading("Controls");
            ui.add(egui::Slider::new(freq, 0.0..=MAXHZ).text("Freq"));
            ui.add(egui::Slider::new(q, 0.1..=10.0).text("Q"));
            ui.add(egui::Slider::new(gain, -5.0..=5.0).text("Gain"));
            ui.horizontal(|ui| {
                ui.radio_value(mode, FiltType::LowShelf, "LowShelf");
                ui.radio_value(mode, FiltType::LP, "LP");
                ui.radio_value(mode, FiltType::BP, "BP");
                ui.radio_value(mode, FiltType::HP, "HP");
                ui.radio_value(mode, FiltType::HighShelf, "HighShelf");
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

fn main() {
    let app = App::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
