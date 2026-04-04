// use bincode::{Decode, Encode};
// use serde::{Deserialize, Serialize};

use std::{
    sync::{Arc, atomic::AtomicBool},
    thread::{JoinHandle, spawn},
};

use crossbeam::channel::Receiver;
#[cfg(feature = "pyo3")]
use log::*;
use midir::MidiOutput;
use pyo3::{prelude::*, types::PyFunction};
#[cfg(not(feature = "pyo3"))]
use tracing::*;

use crate::{
    MidiMsg,
    v2::{Api, MidiDev, MidiThreadCtrlMesg, mk_dev},
};

// #[cfg_attr(feature = "pyo3", pyclass)]
// #[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
// #[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct MidiDawThread {
    rift: Arc<Py<PyFunction>>,
    exec_jh: JoinHandle<()>,
    midi_out_jh: JoinHandle<()>,
    pub exit: Arc<AtomicBool>,
    pub api: Api,
}

impl MidiDawThread {
    pub fn new(rift: Arc<Py<PyFunction>>, exit: Arc<AtomicBool>, api: Api) -> Self {
        Self {
            rift,
            exec_jh: spawn(|| {}),
            midi_out_jh: spawn(|| {}),
            exit,
            api,
        }
    }

    // pub fn set_thread(&mut self, exec_jh: JoinHandle<()>) {
    //     self.exec_jh = exec_jh;
    // }

    pub fn is_running(&self) -> bool {
        !self.exec_jh.is_finished()
    }

    pub fn spawn(
        &mut self,
        func: Arc<Py<PyFunction>>,
        func_name: Arc<String>,
        loop_n: usize,
        api: Api,
        recv: Receiver<MidiThreadCtrlMesg>,
    ) {
        self.rift = func.clone();
        let midi_dev = api.device.clone();
        let channel = api.channel.clone().into();

        self.midi_out_jh = spawn(move || {
            loop {
                // let dev = None;

                // let midi_out = MidiOutput::new(&format!("MIDI-DAW-{midi_dev_name}")).unwrap();
                // while dev.is_none() {
                let mut dev = match midi_dev.clone() {
                    MidiDev::Physical(dev_name) => loop {
                        let midi_out = MidiOutput::new(&format!("MIDI-DAW-{dev_name}")).unwrap();

                        if let Some(Ok(dev)) = midi_out
                            .ports()
                            .into_iter()
                            .find(|p| {
                                midi_out
                                    .port_name(p)
                                    .map(|name| name == dev_name)
                                    .unwrap_or(false)
                            })
                            .map(|dev| midi_out.connect(&dev, &dev_name))
                        {
                            // if let Ok(dev) = midi_out.connect(&dev, &midi_dev_name) {
                            break dev;
                            // }
                        }
                    },
                    MidiDev::Virtual(dev_name) => mk_dev(&dev_name).unwrap(),
                };

                // if let Some(dev) = midi_out.find_port_by_id (dev_id.to_string()).clone() {
                //     if let Ok(dev) = midi_out.connect(&dev, &dev_name) {
                //         midi_devs.insert(dev_name, dev);
                //     } else {
                //         warn!("device named \"{dev_name}\" is no longer connected")
                //         continue;
                //     }
                // } else {
                //     warn!("unknown device id \"{dev_id}\"")
                //     continue;
                // }

                // let Some(dev) = dev {
                //     unreachable!("dev should be a \"Some\" value by this point");
                // };

                // poll for msg to send
                while let Ok(recved_midi_msg) = recv.try_recv() {
                    // match midi_msg {
                    // let (dev_name, msg) = midi_msg;
                    //
                    // if midi_devs.contains_key(&dev_name) => {
                    //     // send messages
                    //     let Some(dev) = midi_devs.get_mut(&dev_name) else {
                    //         error!(
                    //             "an error occured finding the midi device with the name \"{dev_name}\""
                    //         );
                    //         // eprintln!("an error occured finding the midi device with the name \"{dev_name}\"");
                    //         continue;
                    //     };
                    let msg = match recved_midi_msg {
                        MidiMsg::PlayNote {
                            note,
                            velocity,
                            duration: _,
                        } => midi_msg::MidiMsg::ChannelVoice {
                            channel,
                            msg: midi_msg::ChannelVoiceMsg::NoteOn { note, velocity },
                        },
                        MidiMsg::StopNote { note } => midi_msg::MidiMsg::ChannelVoice {
                            channel,
                            msg: midi_msg::ChannelVoiceMsg::NoteOff {
                                note,
                                velocity: 100,
                            },
                        },
                        MidiMsg::PitchBend { bend } => midi_msg::MidiMsg::ChannelVoice {
                            channel,
                            msg: midi_msg::ChannelVoiceMsg::PitchBend { bend },
                        },
                        MidiMsg::CC { control, value } => midi_msg::MidiMsg::ChannelVoice {
                            channel,
                            msg: midi_msg::ChannelVoiceMsg::ControlChange {
                                control: midi_msg::ControlChange::CC { control, value },
                            },
                        },
                    };

                    if let Err(e) = dev.send(&msg.to_midi()) {
                        error!("midi output failed with error {e}");
                        // eprintln!("midi output failed with error {e}");
                        break;
                    }

                    // if let Err(e) = responce_dev.send()
                    // } else {
                    //     error!("the requested midi device, \"{dev_name}\", is not connected.");
                    //     // eprintln!("the requested midi device, \"{dev_name}\", is not connected.");
                    //     error!("known devs = {:?}", midi_devs.keys());
                    // }
                    // }
                }
            }
        });

        self.exec_jh = if loop_n == 0 {
            // let func = func.clone();
            // let api = api.clone();

            spawn(move || {
                Python::initialize();

                Python::attach(move |py| {
                    // let f = {
                    let api = api.into_pyobject(py).unwrap();

                    //     Arc::new(move || func.call1(py, (&api,)))
                    // };

                    loop {
                        // if let Err(e) = f() {
                        if let Err(e) = func.call1(py, (&api,)) {
                            println!("running custom: {func_name}, resulted in error, {e}");
                            break;
                        }
                    }
                })
            })
        } else {
            // let func = func.clone();
            // let api = api.clone();

            spawn(move || {
                Python::initialize();

                Python::attach(move |py| {
                    // let loop_f = {
                    let api = api.into_pyobject(py).unwrap();

                    // Arc::new(move || {
                    for _ in 0..loop_n {
                        if let Err(e) = func.call1(py, (&api,)) {
                            println!("running custom: {func_name}, resulted in error, {e}");
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
