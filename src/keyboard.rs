
use std::convert::Into;
use device_query::{DeviceState, DeviceQuery, Keycode};
use crate::units::{Samples, Hz, Cent, Sec};
use crate::module::*;
use crate::loader::Loader;

pub struct Keyboard {
    dev: DeviceState,
    keymap: Vec<(Keycode, f64)>,
    poll_time: u64,

    oldkeys: Vec<Keycode>,
    timer: u64,
    val: f64,
    oct: f64,
    gate: bool,
    quit: bool,
}

const OCT: f64 = 1200.0;
const CNOTE: f64 = 300.0;
const MAX_OCT: f64 = CNOTE + 3.0 * OCT;
const MIN_OCT: f64 = CNOTE - 4.0 * OCT;

fn has_control(k: &Keycode, keys: &Vec<Keycode>) -> bool {
    (keys.contains(&Keycode::LControl) || keys.contains(&Keycode::RControl)) && keys.contains(k)
}

fn make_keymap() -> Vec<(Keycode, f64)> {
    vec![
        (Keycode::A,    0.0 * 100.0), // C
          (Keycode::W,  1.0 * 100.0),
        (Keycode::S,    2.0 * 100.0),
          (Keycode::E,  3.0 * 100.0),
        (Keycode::D,    4.0 * 100.0),
        (Keycode::F,    5.0 * 100.0),
          (Keycode::T,  6.0 * 100.0),
        (Keycode::G,    7.0 * 100.0),
          (Keycode::Y,  8.0 * 100.0),
        (Keycode::H,    9.0 * 100.0),
          (Keycode::U, 10.0 * 100.0),
        (Keycode::J,   11.0 * 100.0),
        (Keycode::K,   12.0 * 100.0), // C
          (Keycode::O, 13.0 * 100.0),
        (Keycode::L,   14.0 * 100.0), // C
          (Keycode::P, 15.0 * 100.0),
        (Keycode::Semicolon, 16.0 * 100.0),
        (Keycode::Apostrophe, 17.0 * 100.0),
    ]
}

fn new_keys(keys: &Vec<Keycode>, old: &Vec<Keycode>) -> Vec<Keycode> {
    let keep = vec![
        Keycode::LControl,
        Keycode::RControl,
        Keycode::LShift,
        Keycode::RShift,
        // probably want more ....
    ];

    // keep all keys that arent in the old list, or are in the keep list
    keys.iter().filter(|key|
            !old.iter().any(|oldk| oldk == *key)
            || keep.iter().any(|keepk| keepk == *key)
        ).cloned().collect()
}

impl Keyboard {
    pub fn from_cmd(args: &Vec<&str>) -> Result<ModRef, String> {
        if args.len() != 2 {
            return Err(format!("usage: {} polltime", args[0]));
        }
        let t = parse::<f64>("polltime", args[1])?;
        Ok( modref_new(Self::new(Sec(t))) )
    }

    pub fn new(poll: impl Into<Samples>) -> Self {
        println!("  W E   T Y U   O P   ");
        println!(" A S D F G H J K L ; '");
        println!("");
        println!("Z - oct down, X - oct up");
        println!("Hit [Esc] to exit");
        let Samples(time) = poll.into();
        Keyboard {
            dev: DeviceState::new(),
            keymap: make_keymap(),
            oldkeys: Vec::new(),
            poll_time: time,
            timer: time,
            val: 0.0,
            oct: CNOTE,
            gate: false,
            quit: false,
        }
    }
}

fn get_note(keymap: &Vec<(Keycode, f64)>, keys: &Vec<Keycode>) -> Option<f64> {
    for (code, val) in keymap.iter() {
        if keys.contains(&code) { return Some(*val); }
    }
    return None;
}

impl Module for Keyboard {
    fn get_terminals(&self) -> (Vec<TerminalDescr>, Vec<TerminalDescr>) {
        (vec![],
         vec!["out".to_string(),
              "gate".to_string(),
              "quit".to_string()])
    }

    fn get_output(&self, idx: usize) -> Option<f64> {
        if idx == 0 { return Some(self.val); }
        if idx == 1 { return Some(if self.gate { 1.0 } else {0.0}); }
        if idx == 2 { return Some(if self.quit { 1.0 } else {0.0}); }
        return None;
    }

    fn set_input(&mut self, _idx: usize, _value: f64) {
        unreachable!();
    }

    fn advance(&mut self) -> bool {
        if self.timer != 0 {
            self.timer -= 1;
            return !self.quit;
        }
        self.timer = self.poll_time;

        let newkeys = self.dev.get_keys();
        let keys = new_keys(&newkeys, &self.oldkeys);

        if keys.contains(&Keycode::Escape) { self.quit = true; }
        if has_control(&Keycode::C, &keys) { self.quit = true; }
        if keys.contains(&Keycode::Z) && self.oct > MIN_OCT {
            self.oct -= OCT;
            //println!("octave {}", self.oct);
        }
        if keys.contains(&Keycode::X) && self.oct < MAX_OCT {
            self.oct += OCT;
            //println!("octave {}", self.oct);
        }

        if let Some(note) = get_note(&self.keymap, &keys) {
            // play a new note, even if old one is still pressed
            self.gate = true;
            let Hz(freq) = Cent(self.oct + note).into();
            self.val = freq;
            //println!("cent {} freq {}", (self.oct + note)/100.0, freq);
        } else {
            if let Some(_oldnote) = get_note(&self.keymap, &self.oldkeys) {
                // keep the gate on because an old note is still pressed
                self.gate = true;
            } else {
                self.gate = false;
            }
        }
        self.oldkeys = newkeys;

        return !self.quit;
    }
}

pub fn init(l: &mut Loader) {
    l.register("keyboard", Keyboard::from_cmd);
}

