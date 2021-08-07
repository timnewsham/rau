//use std::time::Duration;
//use std::thread::sleep;
use std::cmp;
use std::fs::File;
use std::sync::mpsc;
use wav::{self, bit_depth::BitDepth};
use eframe::{egui, epi};
use egui::{Color32, NumExt};
use egui::widgets::plot::{Line, Values, Value, Plot};
use cpal::{BufferSize, StreamConfig, SampleRate};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

//const FSAMP: f64 = 44100.0;
const FSAMP: f64 = 48000.0;

struct Sample {
    left: f64,
    right: f64,
}

// XXX dummy out tx for now because cpal audio triggers a crash in egui right now
// on windows due to some COM stuff that needs to be fixed in egui.
struct App {
    tx: usize, // XXX mpsc::SyncSender<(f32, f32)>,
    samples: Vec<Sample>,
    time: f64,
}

#[allow(dead_code)]
fn open_speaker() -> mpsc::SyncSender<(f32, f32)> {
    let host = cpal::default_host();
    let dev = host.default_output_device().expect("cant get audio device");
    let cfg = StreamConfig{
        channels: 2,
        sample_rate: SampleRate(FSAMP as u32),
        buffer_size: BufferSize::Default,
    };
    let (tx, rx) = mpsc::sync_channel(64); // XXX parameter

    let pump_func = move |dat: &mut [f32], _: &cpal::OutputCallbackInfo| {
            for n in (0..dat.len()).step_by(2) {
                let (r,l) = rx.recv().unwrap_or((0.0, 0.0));
                dat[n] = r;
                dat[n+1] = l;
            }
        };
    let err_func = |err| { eprintln!("audio output error: {}", err); };
    let stream = dev.build_output_stream(&cfg, pump_func, err_func).expect("cant open audio");
    stream.play().unwrap();
    tx
}

impl App {
    fn from_file(path: &str) -> Self {
        let tx = 0; // = open_speaker();
        let mut inp = File::open(path).expect("couldnt open file");
        let (_hdr, dat) = wav::read(&mut inp).expect("couldn't read samples");
        let mut samples = Vec::new();
        if let BitDepth::Sixteen(vs) = dat {
            for i in (0..vs.len()).step_by(2) {
                let right = (vs[i] as f64) / 32768.0;
                let left = (vs[i+1] as f64) / 32768.0;
                samples.push(Sample{ left: left, right: right } );
            }
        } else {
            panic!("wrong format");
        }

        App {
            tx: tx,
            samples: samples,
            time: 0.0,
        }
    }

    fn max_time(&self) -> f64 {
        self.samples.len() as f64 / FSAMP
    }
}

fn phase_curve(_tx: &usize /*mpsc::SyncSender<(f32, f32)>*/, samps: &Vec<Sample>, from_t: f64, to_t: f64) -> Line {
    let from = (from_t * FSAMP) as usize;
    let to = cmp::min((to_t * FSAMP) as usize, samps.len());
    let dat = (from..to).map(|i| {
            let Sample{left: l, right: r} = samps[i];
            //tx.send((l as f32, r as f32)).expect("failed to send to audio");
            let mid = 0.5 * (l + r);
            let side = 0.5 * (l - r);
            Value::new(side, mid)
        });
        // XXX lines for now, but perhaps better with points?
        Line::new(Values::from_values_iter(dat))
            .color(Color32::from_rgb(100, 200, 100))
            .name("FreqResponse")
    }


impl epi::App for App {
    fn name(&self) -> &str { "Phase Fun" }
    //fn setup(&mut self, _ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>, _storage: Option<&dyn epi::Storage>) { }
    //fn save(&mut self, _storage: &mut dyn epi::Storage) { }
    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let maxt = self.max_time();
        let Self { tx, samples, time } = self;
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

            let curve = phase_curve(&*tx, samples, starttime, endtime);
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
    let app = App::from_file("test.wav");
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
