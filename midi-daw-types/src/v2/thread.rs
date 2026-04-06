// use bincode::{Decode, Encode};
// use serde::{Deserialize, Serialize};

use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{spawn, JoinHandle},
};

use crossbeam::channel::Receiver;
#[cfg(feature = "pyo3")]
use log::*;
use midir::MidiOutput;
use pyo3::{prelude::*, types::PyFunction};
#[cfg(not(feature = "pyo3"))]
use tracing::*;

use crate::{
    v2::{mk_dev, Api, Func, MidiDev, MidiThreadCtrlMesg},
    MidiMsg,
};

// #[cfg_attr(feature = "pyo3", pyclass)]
// #[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
// #[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct MidiDawThread {
    // rift: Arc<Py<PyFunction>>,
    exec_jh: JoinHandle<()>,
    midi_out_jh: JoinHandle<()>,
    pub exit: Arc<AtomicBool>,
    pub api: Api,
    thread_name: String,
}

impl MidiDawThread {
    pub fn new(
        /* rift: Arc<Py<PyFunction>>, */ thread_name: String,
        exit: Arc<AtomicBool>,
        api: Api,
    ) -> Self {
        Self {
            // rift,
            exec_jh: spawn(|| {}),
            midi_out_jh: spawn(|| {}),
            exit,
            api,
            thread_name,
        }
    }

    // pub fn set_thread(&mut self, exec_jh: JoinHandle<()>) {
    //     self.exec_jh = exec_jh;
    // }

    pub fn is_running(&self) -> bool {
        !self.exec_jh.is_finished()
    }

    // pub fn spawn_midi(&mut self, api: Api, recv: Receiver<MidiThreadCtrlMesg>) {
    //     let midi_dev = api.device.clone();
    //     let channel = api.channel.into();
    //
    //     self.midi_out_jh = spawn(move || {
    //         loop {
    //             let mut dev = match midi_dev.clone() {
    //                 MidiDev::Physical(dev_name) => loop {
    //                     let midi_out = MidiOutput::new(&format!("MIDI-DAW-{dev_name}")).unwrap();
    //
    //                     if let Some(Ok(dev)) = midi_out
    //                         .ports()
    //                         .into_iter()
    //                         .find(|p| {
    //                             midi_out
    //                                 .port_name(p)
    //                                 .map(|name| name == dev_name)
    //                                 .unwrap_or(false)
    //                         })
    //                         .map(|dev| midi_out.connect(&dev, &dev_name))
    //                     {
    //                         // if let Ok(dev) = midi_out.connect(&dev, &midi_dev_name) {
    //                         break dev;
    //                         // }
    //                     }
    //                 },
    //                 MidiDev::Virtual(dev_name) => {
    //                     println!("making virtual device named, \"{dev_name}\"");
    //                     mk_dev(&dev_name).unwrap()
    //                 }
    //             };
    //
    //             // poll for msg to send
    //             while let Ok(recved_midi_msg) = recv.recv() {
    //                 // println!("got message: {recved_midi_msg:?}");
    //                 let msg = match recved_midi_msg {
    //                     MidiMsg::PlayNote {
    //                         note,
    //                         velocity,
    //                         duration: _,
    //                     } => midi_msg::MidiMsg::ChannelVoice {
    //                         channel,
    //                         msg: midi_msg::ChannelVoiceMsg::NoteOn { note, velocity },
    //                     },
    //                     MidiMsg::StopNote { note } => midi_msg::MidiMsg::ChannelVoice {
    //                         channel,
    //                         msg: midi_msg::ChannelVoiceMsg::NoteOff {
    //                             note,
    //                             velocity: 100,
    //                         },
    //                     },
    //                     MidiMsg::PitchBend { bend } => midi_msg::MidiMsg::ChannelVoice {
    //                         channel,
    //                         msg: midi_msg::ChannelVoiceMsg::PitchBend { bend },
    //                     },
    //                     MidiMsg::CC { control, value } => midi_msg::MidiMsg::ChannelVoice {
    //                         channel,
    //                         msg: midi_msg::ChannelVoiceMsg::ControlChange {
    //                             control: midi_msg::ControlChange::CC { control, value },
    //                         },
    //                     },
    //                 };
    //
    //                 if let Err(e) = dev.send(&msg.to_midi()) {
    //                     error!("midi output failed with error {e}");
    //                     // eprintln!("midi output failed with error {e}");
    //                     break;
    //                 }
    //             }
    //         }
    //     });
    // }

    pub fn spawn_exec(
        &mut self,
        // func: Arc<Py<PyFunction>>,
        func: Arc<Func>,
        func_name: Arc<String>,
        loop_n: usize,
        api: Api,
        // recv: Receiver<MidiThreadCtrlMesg>,
    ) {
        // self.rift = func.clone();
        let exit = self.exit.clone();

        self.exec_jh = if loop_n == 0 {
            // let func = func.clone();
            // let api = api.clone();
            println!(
                "about to spawn loop theads for thread: {}",
                self.thread_name
            );

            spawn(move || {
                Python::initialize();

                Python::attach(move |py| {
                    // let f = {
                    let api = api.into_pyobject(py).unwrap();

                    //     Arc::new(move || func.call1(py, (&api,)))
                    // };

                    while exit.load(Ordering::Relaxed) {
                        // if let Err(e) = f() {
                        // if let Err(e) = func.call1(py, (&api,)) {
                        if let Err(e) = match func.deref() {
                            Func::PyF(func) => func.call1(py, (&api,)),
                            Func::PyCF(func) => func.call1(py, (&api,)),
                            Func::PyAny(func) => func.call1(py, (&api,)),
                        } {
                            println!("running custom: {func_name}, resulted in error, {e}");
                            break;
                        }
                    }
                })
            })
        } else {
            // let func = func.clone();
            // let api = api.clone();
            println!(
                "about to spawn non-loop theads for thread: {}",
                self.thread_name
            );

            spawn(move || {
                Python::initialize();

                Python::attach(move |py| {
                    // let loop_f = {
                    let api = api.into_pyobject(py).unwrap();

                    // Arc::new(move || {
                    for _ in 0..loop_n {
                        // if let Err(e) = func.call1(py, (&api,)) {
                        if let Err(e) = match func.deref() {
                            Func::PyF(func) => func.call1(py, (&api,)),
                            Func::PyCF(func) => func.call1(py, (&api,)),
                            Func::PyAny(func) => func.call1(py, (&api,)),
                        } {
                            println!("running custom: {func_name}, resulted in error, {e}");
                            break;
                        }
                        if exit.load(Ordering::Relaxed) {
                            break;
                        }
                    }
                    //     })
                    // };
                    //
                    // loop_f()
                })
            })
        };
    }
}
