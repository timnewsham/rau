
use std::convert::Into;
use std::sync::mpsc;
use cpal::{BufferSize, StreamConfig, SampleRate};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

use crate::units::{Samples, SAMPLE_RATE};
use crate::gen::Gen;
use crate::module;

pub struct Speaker {
    tx: mpsc::SyncSender<(f32, f32)>,
    lvalue: f32,
    rvalue: f32,

    #[allow(dead_code)]
    stream: cpal::Stream, // held until speaker is discarded
}

#[allow(dead_code)]
impl Speaker {
    pub fn new() -> Self {
        let host = cpal::default_host();
        let dev = host.default_output_device().expect("cant get audio device");
        let cfg = StreamConfig{
            channels: 2,
            sample_rate: SampleRate(SAMPLE_RATE as u32),
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

        Speaker{ tx: tx, rvalue: 0.0, lvalue: 0.0, stream: stream }
    }

    pub fn record(&mut self, gen: &mut impl Gen, time: impl Into<Samples>) {
        let samples : Samples = time.into();
        for _ in 1 .. samples.0 {
            let v = gen.gen() as f32;
            self.tx.send((v, v)).unwrap();
            gen.advance();
        }
    }
}

impl module::Module for Speaker {
    fn get_terminals(&self) -> (Vec<module::TerminalDescr>, Vec<module::TerminalDescr>) {
        (vec!["left".to_string(),
              "right".to_string()], 
         vec![])
    }

    fn get_output(&self, _idx: usize) -> Option<f64> {
        unreachable!();
    }

    fn set_input(&mut self, idx: usize, value: f64) {
        if idx == 0 { self.lvalue = value as f32; }
        if idx == 1 { self.rvalue = value as f32; }
    }

    fn advance(&mut self) {
        self.tx.send((self.lvalue, self.rvalue)).unwrap();
    }
}
