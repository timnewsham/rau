
use std::convert::Into;
use std::sync::mpsc;
use cpal::{BufferSize, StreamConfig, SampleRate};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

use crate::units::{Samples, SAMPLE_RATE};
use crate::module::*;
use crate::loader::Loader;

pub struct Speaker {
    tx: mpsc::SyncSender<(f32, f32)>,
    lvalue: f32,
    rvalue: f32,

    #[allow(dead_code)]
    stream: cpal::Stream, // held until speaker is discarded
}

#[allow(dead_code)]
impl Speaker {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 1 {
            return Err(format!("usage: {}", args[0]));
        }
        Ok( modref_new(Self::new()) )
    }

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

    pub fn record(&mut self, m: &mut impl Module, outp: &str, time: impl Into<Samples>) -> Result<(), String> {
        let out_idx = output_idx(m, "module", outp)?;
        let Samples(samples) = time.into();
        for _ in 1 .. samples {
            m.advance();
            let v = m.get_output(out_idx).ok_or("can't read gen output")? as f32;
            self.tx.send((v, v)).unwrap();
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
        if idx == 0 { self.lvalue = value as f32; }
        if idx == 1 { self.rvalue = value as f32; }
    }

    fn advance(&mut self) -> bool {
        self.tx.send((self.lvalue, self.rvalue)).unwrap();
        return true;
    }
}

pub fn init(l: &mut Loader) {
    l.register("speaker", Speaker::from_cmd);
}
