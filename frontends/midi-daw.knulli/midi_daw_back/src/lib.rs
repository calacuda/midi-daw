#![feature(lock_value_accessors)]
use midi_daw::server::{BPQ, Tempo};
use midi_daw_types::{MidiChannel, MidiMsg, MidiReqBody, NoteDuration};
use pyo3::prelude::*;
use reqwest::{
    StatusCode,
    blocking::{Client, get},
};
use rustc_hash::FxHashMap;
use serde_json::Value;
use std::{
    sync::{Arc, RwLock},
    thread::{JoinHandle, spawn},
};
use tungstenite::connect;

pub const N_STEPS: usize = 16;

pub type Sequences = Arc<RwLock<FxHashMap<String, Sequence>>>;
pub type PlayingSequences = Arc<RwLock<Vec<String>>>;

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
    pub tempo: Tempo,
    pub bpq: BPQ,
    pub pusles: Arc<RwLock<usize>>,
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
    pub dev: String,
    pub channel: MidiChannel,
    /// (Option<(midi_note, velocity)>, cmd_1, cmd_2)
    pub steps: [(Option<(u8, u8)>, Option<Cmd>, Option<Cmd>); N_STEPS],
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
    _jh: JoinHandle<()>,
    pub midi_handler: Arc<RwLock<MidiOutHandlerTarget>>,
    pub sequences: Sequences, // Arc<RwLock<FxHashMap<String, Sequence>>>,
    pub playing: PlayingSequences,
}

// impl MidiOut {
//     fn send_midi(&mut self, message: MidiMsg) {
//         match self.midi_handler.read().map(|lock| lock.clone()) {
//             Ok(MidiOutHandlerTarget::Remote { ref base_url }) => {
//                 // TODO: mk web req
//             }
//             Ok(MidiOutHandlerTarget::Local(ref _conf)) => {
//                 // TODO: send to local theread
//             }
//             Err(e) => {
//                 println!("failed to read midi_handler {e}");
//             }
//         }
//     }
// }

// TODO: make a thread that plays for each playing sequence OR make one thread that plays them all

// TODO: make the displaying sequences rotatable by a modifier key and the d-pad

#[pymethods]
impl MidiOut {
    #[new]
    pub fn new() -> Self {
        let sequences = Arc::new(RwLock::new({
            let mut map = FxHashMap::default();
            map.insert(
                "Example-1".into(),
                Sequence::new("Set2Device", MidiChannel::Ch1),
            );
            map.insert(
                "Example-2".into(),
                Sequence::new("Set2Device", MidiChannel::Ch1),
            );
            map.insert(
                "Example-3".into(),
                Sequence::new("Set2Device", MidiChannel::Ch1),
            );

            map
        }));
        let playing = Arc::new(RwLock::new(Vec::default()));
        let base_url: String = "http://127.0.0.1:8080".into();
        let midi_handler = Arc::new(RwLock::new(MidiOutHandlerTarget::Remote {
            base_url: base_url.clone(),
        }));

        Self {
            _jh: spawn({
                let sequences = sequences.clone();
                let playing = playing.clone();
                // let midi_handler = midi_handler.clone();

                move || {
                    sequence_thread(
                        sequences, playing, // midi_handler,
                        base_url,
                    )
                }
            }),
            midi_handler,
            sequences,
            playing,
        }
    }

    fn change_midi_out_handler(&mut self, handler: MidiOutHandlerTarget) {
        _ = self.midi_handler.replace(handler);
    }

    /// sets the tempo
    pub fn set_tempo(&mut self, tempo: f64) {
        match self.midi_handler.read().map(|lock| lock.clone()) {
            Ok(MidiOutHandlerTarget::Remote { ref base_url }) => {
                // mk api call
                let client = Client::new();

                if let Err(e) = client.post(format!("{base_url}/tempo")).json(&tempo).send() {
                    println!("posting a message failed with: {e}");
                }
            }
            Ok(MidiOutHandlerTarget::Local(ref _conf)) => {}
            Err(e) => {
                println!("failed to read midi_handler {e}");
            }
        }
    }

    /// returns the current tempo
    pub fn get_tempo(&mut self) -> f64 {
        match self.midi_handler.read().map(|lock| lock.clone()) {
            Ok(MidiOutHandlerTarget::Remote { ref base_url }) => {
                let Ok(req) = get(format!("{base_url}/midi")) else {
                    return 0.0;
                };
                let Ok(body) = req.text() else {
                    return 0.0;
                };

                if let Ok(Value::Number(num)) = serde_json::from_str(&body)
                // && num.is_f64()
                {
                    num.as_f64().unwrap_or(0.0)
                } else {
                    0.0
                }
            }
            Ok(MidiOutHandlerTarget::Local(ref conf)) => {
                if let Ok(tempo) = conf.tempo.read() {
                    *tempo
                } else {
                    0.0
                }
            }
            Err(e) => {
                println!("failed to read midi_handler {e}");
                0.0
            }
        }
    }

    fn list_devs(&mut self) -> Vec<String> {
        match self.midi_handler.read().map(|lock| lock.clone()) {
            Ok(MidiOutHandlerTarget::Remote { ref base_url }) => {
                // mk web req
                let Ok(req) = get(format!("{base_url}/midi")) else {
                    return Vec::new();
                };
                let Ok(body) = req.text() else {
                    return Vec::new();
                };

                if let Ok(Value::Array(arr)) = serde_json::from_str(&body) {
                    arr.into_iter().map(|elm| elm.to_string()).collect()
                } else {
                    Vec::new()
                }
            }
            Ok(MidiOutHandlerTarget::Local(ref _conf)) => Vec::new(),
            Err(e) => {
                println!("failed to read midi_handler {e}");

                Vec::new()
            }
        }
    }

    fn rest(&self, time: NoteDuration) {
        match self.midi_handler.read().map(|lock| lock.clone()) {
            Ok(MidiOutHandlerTarget::Remote { ref base_url }) => {
                // mk req
                let client = Client::new();

                if let Err(e) = client.post(format!("{base_url}/rest")).json(&time).send() {
                    println!("posting a message failed with: {e}");
                }
            }
            Ok(MidiOutHandlerTarget::Local(ref _conf)) => {}
            Err(e) => {
                println!("failed to read midi_handler {e}");
            }
        }
    }

    fn rename_seq(&mut self, old_name: String, new_name: String) {
        // TODO: change name in self.sequences
        // TODO: change name in self.playing
    }

    fn set_note(&mut self, seq_name: String, step_num: usize, note: Option<u8>, vel: Option<u8>) {
        _ = self.sequences.write().map(|ref mut map| {
            if let Some(seq) = map.get_mut(&seq_name) {
                match (note, vel) {
                    // Some((old_note, ))
                    (Some(note), Some(vel)) => {
                        seq.steps[step_num].0.replace((note, vel));
                    }
                    (Some(note), None) => {
                        seq.steps[step_num].0.replace((note, vel.unwrap_or(90)));
                    }
                    (None, Some(vel)) => {
                        seq.steps[step_num].0 = seq.steps[step_num].0.map(|(note, _)| (note, vel));
                    }
                    (None, None) => {
                        seq.steps[step_num].0 = None;
                    }
                }
            };
        });
    }

    fn new_seq(&mut self, seq_name: String, dev_name: String, channel: MidiChannel) {
        _ = self
            .sequences
            .write()
            .map(|ref mut map| map.insert(seq_name, Sequence::new(&dev_name, channel)));
    }

    fn get_seq_names(&self) -> Vec<(String, String)> {
        if let Ok(seqs) = self.sequences.read() {
            let mut names: Vec<String> = seqs.keys().map(|name| name.to_owned()).collect();
            names.sort();

            names
                .iter()
                .map(|name| {
                    (
                        name.clone(),
                        seqs.get(name)
                            .map(|seq| seq.dev.clone())
                            .unwrap_or("UNKNOWN".into()),
                    )
                })
                .collect()
        } else {
            Vec::new()
        }
    }

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
                println!("{seq_name}, is not one of; {:?}", seqs.keys());
                None
            }
        } else {
            None
        }
    }

    fn play_seq(&mut self, seq_name: String) {
        if let Ok(mut seqs) = self.playing.write()
            && self
                .sequences
                .read()
                .unwrap()
                .keys()
                .collect::<Vec<_>>()
                .contains(&&seq_name)
        {
            // println!("Playing sequence {seq_name}");
            seqs.push(seq_name);
        } else {
            println!("could not play");
        }
    }

    // fn panic(&self, dev_name: &str) {
    //     // TODO: write this
    // }
}

fn sequence_thread(
    sequences: Sequences,
    playing: PlayingSequences,
    // midi_handler: Arc<RwLock<MidiOutHandlerTarget>>,
    server_url: String,
) -> ! {
    loop {
        let base_url = server_url.clone();
        let ws_url = format!("{}/message-bus", base_url.replace("http://", "ws://"));
        // connect to websocket
        let conn = connect(ws_url.clone());
        // println!("conn {conn:?}");

        // Establish a connection to the WebSocket server
        if let Ok((mut socket, response)) = conn
            && !response.status().is_server_error()
            && !response.status().is_client_error()
        {
            let mut sync_pulses: usize = 0;
            let mut note_threads = Vec::new();

            // while connected to websocket ...
            while let Ok(msg) = socket.read() {
                if msg.is_binary() {
                    // println!("bin message");

                    if let (Ok(sequences), Ok(playing)) = (sequences.read(), playing.read())
                        && sync_pulses % 12 == 0
                    {
                        let step_num = (sync_pulses / 12) % N_STEPS;

                        note_threads.clear();
                        // play notes
                        for seq_name in playing.iter() {
                            if let Some(seq) = sequences.get(seq_name) {
                                // TODO: do before note cmd_stuff

                                // play note
                                if let Some((note, velocity)) = seq.steps[step_num].0 {
                                    let jh = spawn({
                                        let midi_dev = seq.dev.clone();
                                        let channel = seq.channel.clone();
                                        let url = format!("{base_url}/midi");

                                        move || {
                                            // mk req
                                            let client = Client::new();
                                            let msg = MidiMsg::PlayNote {
                                                note,
                                                velocity,
                                                duration: NoteDuration::Sn(1),
                                            };
                                            let midi_req_body = MidiReqBody {
                                                midi_dev,
                                                channel,
                                                msg,
                                            };

                                            if let Err(e) =
                                                client.post(url).json(&midi_req_body).send()
                                            {
                                                println!("posting a message failed with: {e}");
                                            }
                                        }
                                    });

                                    note_threads.push(jh);
                                }

                                // TODO: do after note cmd_stuff
                            }
                        }
                    }

                    sync_pulses += 1;
                }
            }
        } else {
            println!("failed to connect to {ws_url}");
            // delay
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

// /// Formats the sum of two numbers as string.
// #[pyfunction]
// fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
//     Ok((a + b).to_string())
// }

/// A Python module implemented in Rust.
#[pymodule]
fn midi_daw_back(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<MidiOut>();

    Ok(())
}
