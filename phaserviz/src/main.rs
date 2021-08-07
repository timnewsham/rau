use num_complex::Complex;
use eframe::{egui, epi};
use egui::{remap, Color32};
use egui::widgets::plot::{Line, Values, Value, Plot};

use rau::units::{Hz, RadPS, MAXHZ};

const MINDB: f64 = -30.0;

fn to_db(pow: f64) -> f64 {
    let db = 10.0 * pow.log(10.0);
    if db.is_nan() || db < MINDB { MINDB } else { db }
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Mode { MixedPhase, MixedDB, AllpassPhase, AllpassDB }

struct App {
    g: f64,
    mode: Mode,
}

fn allpass(g: f64, z: Complex<f64>) -> Complex<f64> {
    return (z*g + 1.0) / (z + g)
}

// frequency response at radian frequency w
fn response(gs: &Vec<f64>, z: Complex<f64>) -> Complex<f64> {
    let r = gs.iter().map(|g| allpass(*g, z)).product();
    //let r = allpass(g/8.0, z) + allpass(g/4.0, z) + allpass(g/2.0, z) + allpass(g/1.0, z);
    //let r = allpass(g, z);
    return r;
}

fn curve(mode: Mode, gs: &Vec<f64>) -> Line {
    let n = 512;
    let dat = (0..n).map(|i| {
        let freq = remap(i as f64, 0.0..=(n as f64), 0.0..=MAXHZ);
        let RadPS(w) = Hz(freq).into();
        let z = Complex::new(w.cos(), w.sin());
        let r = response(gs, z);
        match mode {
            Mode::AllpassPhase => Value::new(freq, r.arg()),
            Mode::AllpassDB    => Value::new(freq, to_db(r.norm_sqr())),
            Mode::MixedPhase   => Value::new(freq, (z-r).arg()),
            Mode::MixedDB      => Value::new(freq, to_db((z- r).norm_sqr())),
        }
    });
    Line::new(Values::from_values_iter(dat))
}

impl Default for App {
    fn default() -> Self {
        App { 
            g: 0.5,
            mode: Mode::MixedDB,
        }
    }
}

impl epi::App for App {
    fn name(&self) -> &str { "Filter Fun" }
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let Self { g, mode } = self;
        egui::TopBottomPanel::top("Filter Fun").show(ctx, |ui| {
            ui.heading("Controls");
            ui.add(egui::Slider::new(g, -0.99..=0.99).text("g"));
            ui.horizontal(|ui| {
                ui.radio_value(mode, Mode::AllpassPhase, "allpass phase");
                ui.radio_value(mode, Mode::AllpassDB, "allpass dB");
                ui.radio_value(mode, Mode::MixedPhase, "mixed phase");
                ui.radio_value(mode, Mode::MixedDB, "mixed dB");
            });
            let plot = Plot::new("response")
                .line(curve(*mode, &vec![*g/8.0, *g/4.0, *g/2.0, *g/1.0])
                        .color(Color32::from_rgb(100,200,100))
                        .name("All"))
                .line(curve(*mode, &vec![*g/8.0])
                        .color(Color32::from_rgb(200,200,100))
                        .name("g1"))
                .line(curve(*mode, &vec![*g/4.0])
                        .color(Color32::from_rgb(200,100,100))
                        .name("g2"))
                .line(curve(*mode, &vec![*g/2.0])
                        .color(Color32::from_rgb(200,100,200))
                        .name("g3"))
                .line(curve(*mode, &vec![*g/1.0])
                        .color(Color32::from_rgb(100,100,200))
                        .name("g4"))
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
