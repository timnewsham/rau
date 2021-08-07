use num_complex::Complex;
use eframe::{egui, epi};
use egui::{remap, Color32};
use egui::widgets::plot::{Line, Values, Value, Plot};

use rau::units::{Hz, RadPS, MAXHZ, Sec, Samples};

const MINDB: f64 = -30.0;

fn to_db(pow: f64) -> f64 {
    let db = 10.0 * pow.log(10.0);
    if db.is_nan() || db < MINDB { MINDB } else { db }
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Mode { MixedPhase, MixedDB, DelayPhase, DelayDB }

struct App {
    delay: f64,
    forw: f64,
    mode: Mode,
}

fn response(sampdelay: f64, z: Complex<f64>) -> Complex<f64> {
    return 0.5 * z.powf(-sampdelay);
}

fn curve(mode: Mode, forw: f64, delay: f64) -> Line {
    let n = 512;
    let Samples(sampdelay_) = Sec(delay).into();
    let sampdelay = sampdelay_ as f64;
    
    let dat = (0..n).map(|i| {
        let freq = remap(i as f64, 0.0..=(n as f64), 0.0..=MAXHZ);
        let RadPS(w) = Hz(freq).into();
        let z = Complex::new(w.cos(), w.sin());
        let r = response(sampdelay, z);
        match mode {
            Mode::DelayPhase => Value::new(freq, r.arg()),
            Mode::DelayDB    => Value::new(freq, to_db(r.norm_sqr())),
            Mode::MixedPhase => Value::new(freq, (forw * z - r).arg()),
            Mode::MixedDB    => Value::new(freq, to_db((forw * z - r).norm_sqr())),
        }
    });
    Line::new(Values::from_values_iter(dat))
}

impl Default for App {
    fn default() -> Self {
        App { 
            delay: 0.1e-3,
            forw: 0.5,
            mode: Mode::MixedDB,
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str { "Filter Fun" }
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let Self { forw, delay, mode } = self;
        egui::TopBottomPanel::top("Filter Fun").show(ctx, |ui| {
            ui.heading("Controls");
            ui.add(egui::Slider::new(delay, 0.1e-3..=1e-3).text("delay"));
            ui.add(egui::Slider::new(forw, 0.1..=1.1).text("feedforward"));
            ui.horizontal(|ui| {
                ui.radio_value(mode, Mode::DelayPhase, "delay phase");
                ui.radio_value(mode, Mode::DelayDB, "delay dB");
                ui.radio_value(mode, Mode::MixedPhase, "mixed phase");
                ui.radio_value(mode, Mode::MixedDB, "mixed dB");
            });
            let plot = Plot::new("response")
                .line(curve(*mode, *forw, *delay)
                        .color(Color32::from_rgb(100,200,100))
                        .name("Response"))
                .view_aspect(1.5)
                //.include_y(MINDB)
                .include_y(-1.0)
                .include_y(1.0)
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
