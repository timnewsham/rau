
use std::convert::{Into, From};
use std::sync::mpsc;
use cpal::{BufferSize, StreamConfig, SampleRate};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};

use crate::units::{Samples, SAMPLE_RATE};
use crate::resampler::ResamplerStereo;
use crate::module::*;

// belongs elsewhere
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub left: f64,
    pub right: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct MidSide {
    pub mid: f64,
    pub side: f64,
}

impl From<Sample> for MidSide {
    fn from(samp: Sample) -> MidSide {
        MidSide { 
            mid: (samp.right + samp.left) / 2.0, // mono mix-down
            side: (samp.right - samp.left) / 2.0,
        }
    }
}

impl From<MidSide> for Sample {
    fn from(ms: MidSide) -> Sample {
        Sample {
            left: ms.mid - ms.side, // 0.5(right + left) - 0.5(right - left) = left
            right: ms.mid + ms.side, // 0.5(right + left) + 0.5(right - left) = right
        }
    }
}

pub trait SamplePlayer {
    fn play(&mut self, sample: Sample);
}

pub struct Speaker {
    tx: mpsc::SyncSender<Sample>,

    #[allow(dead_code)]
    stream: cpal::Stream, // held for reference

    lvalue: f64,
    rvalue: f64,
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
                    match rx.recv() {
                        Err(_) => break,
                        Ok(Sample{ left, right }) => {
                            dat[n] = left as f32;
                            dat[n+1] = right as f32;
                        },
                    }
                }
            };
        let err_func = |err| { eprintln!("audio output error: {}", err); };
        let stream = dev.build_output_stream(&cfg, pump_func, err_func).expect("cant open audio");
        stream.play().expect("error starting audio");

        Speaker{ tx, rvalue: 0.0, lvalue: 0.0, stream }
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

impl SamplePlayer for Speaker {
    fn play(&mut self, sample: Sample) {
        self.tx.send(sample).expect("cant send audio data");
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
        true
    }
}

pub fn init(l: &mut Loader) {
    l.register("speaker", Speaker::from_cmd);
}

pub struct ResamplingSpeaker {
    resampler: ResamplerStereo,
    speaker: Speaker,
}

impl ResamplingSpeaker {
    // Consumes 44.1KHz audio and plays on 48KHz audio speaker
    pub fn new_441_to_480(qsize: usize) -> Self {
        Self {
            resampler: ResamplerStereo::new_441_to_480(),
            speaker: Speaker::new_full(48000.0, qsize),
        }
    }
}

impl SamplePlayer for ResamplingSpeaker {
    fn play(&mut self, sample: Sample) {
        let Self{ resampler, speaker } = self;
        resampler.resample(sample, |s| speaker.play(s));
    }
}

pub type DynSpeaker = Box<dyn SamplePlayer>;

pub fn player_at(rate: u32, qsize: usize) -> DynSpeaker {
    assert!(SAMPLE_RATE == 48000.0);
    match rate {
        44100 => Box::new(ResamplingSpeaker::new_441_to_480(qsize)),
        48000 => Box::new(Speaker::new_full(SAMPLE_RATE, qsize)),
        _ => panic!("unsupported sampling rate"),
    }
}

