use midi_daw::{
    midi::MidiDev,
    server::{BPQ, Tempo},
};
use midi_daw_types::{MidiChannel, MidiMsg, NoteDuration};
use pyo3::prelude::*;
use rustc_hash::{FxHashMap, FxHashMapSeed};
use std::{
    sync::{Arc, RwLock},
    thread::JoinHandle,
};

pub const N_STEPS: usize = 16;

pub trait MidiOutHandler {
    fn set_tempo(&mut self, tempo: f64);
    fn get_tempo(&self) -> f64;
    fn send_midi(&mut self, message: MidiMsg);
    fn list_devs(&mut self) -> Vec<String>;
    fn rest(&self, time: NoteDuration);
}

#[pyclass()]
#[derive(Debug, Clone)]
pub struct LocalMidiOut {
    tempo: Tempo,
    bpq: BPQ,
    pusles: Arc<RwLock<usize>>,
}

#[pyclass()]
#[derive(Debug, Clone)]
pub enum MidiOutHandlerTarget {
    Local(LocalMidiOut),
    Remote { base_url: String },
    // Both {
    //     base_url: String,
    // },
}

#[derive(Debug, Clone, Copy)]
pub enum Cmd {
    None,
}

#[derive(Debug)]
pub struct Sequence {
    dev: String,
    channel: MidiChannel,
    /// (Option<(midi_note, velocity)>, cmd_1, cmd_2)
    steps: [(Option<(u8, u8)>, Option<Cmd>, Option<Cmd>); N_STEPS],
}

impl Sequence {
    pub fn new(dev: &str, channel: MidiChannel) -> Self {
        let steps = [(None, None, None); N_STEPS];

        Self {
            dev: dev.into(),
            channel,
            steps,
        }
    }
}

#[pyclass]
#[derive(Debug)]
pub struct MidiOut {
    /// the join handle for the local output thread
    _jh: Option<JoinHandle<()>>,
    midi_handler: MidiOutHandlerTarget,
    sequences: Arc<RwLock<FxHashMap<String, Sequence>>>,
    playing: Arc<RwLock<Vec<String>>>,
}

impl MidiOut {
    fn send_midi(&mut self, message: MidiMsg) {
        match self.midi_handler {
            MidiOutHandlerTarget::Remote { ref base_url } => {
                // TODO: mk web req
            }
            MidiOutHandlerTarget::Local(ref _conf) => {
                // TODO: send to local theread
            }
        }
    }
}

// TODO: make a thread that plays for each playing sequence OR make one thread that plays them all

// TODO: make the displaying sequences rotatable by a modifier key and the d-pad

#[pymethods]
impl MidiOut {
    #[new]
    pub fn new() -> Self {
        let sequences = Arc::new(RwLock::new({
            let mut map = FxHashMap::default();
            map.insert("Example-1".into(), Sequence::new("None", MidiChannel::Ch1));

            map
        }));

        let playing = Arc::new(RwLock::new(Vec::default()));

        Self {
            _jh: None,
            midi_handler: MidiOutHandlerTarget::Remote {
                base_url: "http://127.0.0.1:8080/".into(),
            },
            sequences,
            playing,
        }
    }

    fn change_midi_out_handler(&mut self, handler: MidiOutHandlerTarget) {
        self.midi_handler = handler;
    }

    /// sets the tempo
    pub fn set_tempo(&mut self, tempo: f64) {
        match self.midi_handler {
            MidiOutHandlerTarget::Remote { ref base_url } => {
                // TODO: mk api call
            }
            MidiOutHandlerTarget::Local(ref _conf) => {}
        }
    }

    /// returns the current tempo
    pub fn get_tempo(&mut self) -> f64 {
        match self.midi_handler {
            MidiOutHandlerTarget::Remote { ref base_url } => {
                // TODO: mk request
                // TODO: parse float
                0.0
            }
            MidiOutHandlerTarget::Local(ref conf) => {
                if let Ok(tempo) = conf.tempo.read() {
                    *tempo
                } else {
                    0.0
                }
            }
        }
    }

    fn list_devs(&mut self) -> Vec<String> {
        match self.midi_handler {
            MidiOutHandlerTarget::Remote { ref base_url } => {
                // TODO: mk web req
                Vec::new()
            }
            MidiOutHandlerTarget::Local(ref _conf) => Vec::new(),
        }
    }

    fn rest(&self, time: NoteDuration) {
        match self.midi_handler {
            MidiOutHandlerTarget::Remote { ref base_url } => {
                // TODO: mk web req
            }
            MidiOutHandlerTarget::Local(ref _conf) => {}
        }
    }

    fn rename_seq(&mut self, old_name: String, new_name: String) {}

    fn set_note(&self, seq_name: String, step_num: usize, note: Option<u8>, vel: Option<u8>) {}

    fn get_seq_row(&self, seq_name: String, step_num: usize) -> Option<[String; 4]> {
        if let Ok(seqs) = self.sequences.read()
            && step_num < N_STEPS
        {
            if let Some(seq) = seqs.get(&seq_name) {
                let (note, _cmd_1, _cmd_2) = seq.steps[step_num];
                let n = note
                    .map(|note| display_midi_note(note.0))
                    .unwrap_or("---".into());
                let vel = note
                    .map(|note| format!("{:->3}", note.1))
                    .unwrap_or("---".into());

                // Some([n, vel, cmd_1, cmd_2])
                Some([n, vel, "----".into(), "----".into()])
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub fn display_midi_note(midi_note: u8) -> String {
    let note_name_i = midi_note % 12;
    let octave = midi_note / 12;

    let note_names = [
        "C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-",
    ];
    let note_name = note_names[note_name_i as usize];

    format!("{note_name}{octave:X}")
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn midi_daw_back(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}
