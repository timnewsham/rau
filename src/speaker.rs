
use std::convert::Into;
use std::sync::mpsc;
use cpal::{BufferSize, StreamConfig, SampleRate};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

use crate::units::{Samples, SAMPLE_RATE};
use crate::module::*;

#[derive(Clone, Copy)]
pub struct Sample {
    pub left: f64,
    pub right: f64,
}

pub struct Speaker {
    tx: mpsc::SyncSender<Sample>,
    lvalue: f64,
    rvalue: f64,

    #[allow(dead_code)]
    stream: cpal::Stream, // held until speaker is discarded
}

impl Speaker {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 1 {
            return Err(format!("usage: {}", args[0]));
        }
        Ok( modref_new(Self::new()) )
    }

    pub fn new() -> Self {
        Self::new_full(SAMPLE_RATE, 64)
    }

    pub fn new_full(fsamp: f64, qsize: usize) -> Self {
        let host = cpal::default_host();
        let dev = host.default_output_device().expect("cant get audio device");
        let cfg = StreamConfig{
            channels: 2,
            sample_rate: SampleRate(fsamp as u32),
            buffer_size: BufferSize::Default,
        };
        let (tx, rx) = mpsc::sync_channel(qsize);

        let pump_func = move |dat: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for n in (0..dat.len()).step_by(2) {
                    let sample: Sample = rx.recv().expect("error receiving audio");
                    dat[n] = sample.left as f32;
                    dat[n+1] = sample.right as f32;
                }
            };
        let err_func = |err| { eprintln!("audio output error: {}", err); };
        let stream = dev.build_output_stream(&cfg, pump_func, err_func).expect("cant open audio");
        stream.play().expect("error starting audio");

        Speaker{ tx: tx, rvalue: 0.0, lvalue: 0.0, stream: stream }
    }

    pub fn play(&mut self, sample: Sample) {
        self.tx.send(sample).expect("cant send audio data");
    }

    pub fn record(&mut self, m: &mut impl Module, outp: &str, time: impl Into<Samples>) -> Result<(), String> {
        let out_idx = m.output_idx("module", outp)?;
        let Samples(samples) = time.into();
        for _ in 1 .. samples {
            m.advance();
            let v = m.get_output(out_idx).ok_or("can't read gen output")?;
            self.play(Sample{ left: v, right: v, });
        }
        Ok(())
    }
}

impl Module for Speaker {
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        (vec!["left".to_string(),
              "right".to_string()],
         vec![])
    }

    fn get_output(&self, _idx: usize) -> Option<f64> {
        unreachable!();
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 { self.lvalue = value; }
        if idx == 1 { self.rvalue = value; }
    }

    fn advance(&mut self) -> bool {
        self.play(Sample{ left: self.lvalue, right: self.rvalue, });
        return true;
    }
}

pub fn init(l: &mut Loader) {
    l.register("speaker", Speaker::from_cmd);
}
