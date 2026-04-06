use crate::{
    v1::note_from_str, v2::thread::MidiDawThread, MidiChannel, MidiDeviceName, MidiMsg,
    NoteDuration,
};
use bincode::{Decode, Encode};
use crossbeam::channel::{unbounded, Receiver, Sender};
use lazy_static::lazy_static;
#[cfg(feature = "pyo3")]
use log::*;
use midir::{os::unix::VirtualOutput, MidiOutput, MidiOutputConnection};
use musical_scales::{PitchClass, Scale, ScaleType};
use pyo3::types::PyCFunction;
#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, types::PyFunction};
use rust_fuzzy_search::fuzzy_search_best_n;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::{
    ffi::CString,
    fmt::Display,
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread::{sleep_until, spawn, JoinHandle},
    time::{Duration, Instant},
};
#[cfg(not(feature = "pyo3"))]
use tracing::*;

// pub mod v2;
pub mod automation;
pub mod thread;

// pub type Scale = Vec<String>;
pub type MidiThreadCtrlMesg = ((MidiDev, MidiChannel), MidiMsg);

lazy_static! {
    /// This is an example for using doc comment attributes
    static ref MIDI_OUT_THREAD_COMS: (Sender<MidiThreadCtrlMesg>, Receiver<MidiThreadCtrlMesg>) = unbounded();
    static ref MIDI_OUT: Sender<MidiThreadCtrlMesg> = MIDI_OUT_THREAD_COMS.0.clone();
    // static ref MIDI_OUT_THREAD: JoinHandle<()> = spawn(midi_out_thread);

}

fn midi_out_thread() {
    // println!("MIDI_OUT_THREAD started");
    // TODO: add midi clock syncing.
    // TODO: add a message that can be sent to here which will register a notification on a
    // specified clock event.
    let mut midi_devs = FxHashMap::<MidiDeviceName, MidiOutputConnection>::default();

    let mut send_to_dev = |midi_msg: MidiThreadCtrlMesg| -> bool {
        match midi_msg {
            (
                (MidiDev::Physical(dev_name) | MidiDev::Virtual(dev_name), channel),
                msg, /*, responce_dev*/
            ) if midi_devs.contains_key(&dev_name) => {
                let Some(dev) = midi_devs.get_mut(&dev_name) else {
                    unreachable!(
                        "an error occured finding the midi device with the name \"{dev_name}\". not retrying"
                    );
                    // eprintln!("an error occured finding the midi device with the name \"{dev_name}\"");
                    // return false;
                };

                let channel = channel.into();

                let msg = match msg {
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

                // println!("sending a message");

                if let Err(e) = dev.send(&msg.to_midi()) {
                    error!("midi output failed with error {e}");
                    // eprintln!("midi output failed with error {e}");
                }

                // println!(
                //     "midi message: {msg:?}, was sent to, {}:{}",
                //     dev_name, channel
                // );

                // if let Err(e) = responce_dev.send()
                true
            }
            ((MidiDev::Virtual(dev_name), _channel), _msg /*, _responce_dev*/)
                if !midi_devs.contains_key(&dev_name) =>
            {
                info!("the requested virtual midi device, \"{dev_name}\", has yet to be made, making now.");
                let Ok(midi_out) = MidiOutput::new("MIDI-DAW-NEW-DEV") else {
                    return false;
                };

                if let Ok(dev) = midi_out.create_virtual(&dev_name)
                    && !midi_devs.contains_key(&dev_name)
                {
                    midi_devs.insert(dev_name, dev);
                } else if midi_devs.contains_key(&dev_name) {
                    info!("device already exists")
                } else {
                    error!("failed to make virtual output device");
                }

                false
            }
            ((MidiDev::Physical(dev_name), _channel), _msg) => {
                error!("the requested physical midi device, \"{dev_name}\", is not connected.");
                error!("known midi devs = {:?}", midi_devs.keys());
                true
            }
            ((MidiDev::Virtual(dev_name), _channel), _msg) => {
                unreachable!(
                    "virtual midi device, \"{dev_name}\", was not found and the code failed to detect that it should be created..."
                )
            }
        }
    };

    loop {
        // poll for msg to send
        while let Ok(midi_msg) = MIDI_OUT_THREAD_COMS.1.try_recv() {
            if !send_to_dev(midi_msg.clone()) {
                send_to_dev(midi_msg);
            }
        }
    }
}

// #[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
// #[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
// #[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Clone, Debug)]
pub struct Api {
    // NOTE: Only Needed if calling multiple functions
    // riffs: Vec<Py<()>>,
    #[pyo3(get, name = "device")]
    // pub device_name: MidiDeviceName,
    pub device: MidiDev,
    #[pyo3(get, set)]
    pub channel: MidiChannel,
    // __threads: Vec<JoinHandle<()>>,
    __coms: Sender<MidiThreadCtrlMesg>,
    __name: String,
    tempo: f64,
    scale: Option<(Scale, u8)>,
}

impl Api {
    fn new(
        dev: MidiDev,
        channel: MidiChannel,
        // __coms: Sender<MidiThreadCtrlMesg>,
        name: String,
        tempo: f64,
    ) -> Self {
        Self {
            device: dev,
            channel,
            // __threads: Vec::new(),
            __coms: MIDI_OUT_THREAD_COMS.0.clone(),
            __name: name,
            tempo,
            scale: None,
        }
    }
}

impl Api {
    pub fn do_rest(&self, dur: NoteDuration, now: Instant) {
        // let now = Instant::now();
        let (mul, denom) = match dur {
            NoteDuration::Wn(n) => (n, 4.0),
            NoteDuration::Hn(n) => (n, 2.0),
            NoteDuration::Qn(n) => (n, 1.0),
            NoteDuration::En(n) => (n, 1.0 / 2.0),
            NoteDuration::Sn(n) => (n, 1.0 / 4.0),
            NoteDuration::Tn(n) => (n, 1.0 / 8.0),
            NoteDuration::S4n(n) => (n, 1.0 / 16.),
        };
        let mul = mul as f64;

        Python::attach(|py| {
            py.detach(|| {
                sleep_until(now + Duration::from_secs_f64(((60.0 / self.tempo) * denom) * mul))
            })
        });
    }
}

#[pymethods]
impl Api {
    // #[new]
    // fn new(dev: MidiDeviceName, channel: MidiChannel) -> Self {
    //     Self {
    //         device: dev,
    //         channel,
    //     }
    // }

    // /// starts playback
    // fn start(&self) {}

    /// plays a note
    #[pyo3(signature = (note, dur = None, vel = None, blocking = None))]
    fn note(
        &self,
        note: String,
        dur: Option<NoteDuration>,
        vel: Option<u8>,
        blocking: Option<bool>,
    ) {
        let now = Instant::now();
        let dur = dur.unwrap_or(NoteDuration::Sn(1));
        // println!(
        //     "playing {note}@{vel:?} for {dur:?}, on: {:?}. blocking? {blocking:?}",
        //     self.device
        // );
        let note =
            if let (Some((scale, octave)), Ok(new_note)) = (&self.scale, note.parse::<usize>()) {
                // println!("new_note: {new_note}");

                if let Ok(note) = scale.idx_to_pitch(new_note - 1) {
                    note.to_midi() + 12 * octave
                } else {
                    note_from_str(note).unwrap_or(0)
                }
            } else {
                note_from_str(note).unwrap_or(0)
            };
        self.__coms.send((
            (self.device.clone(), self.channel),
            MidiMsg::PlayNote {
                note,
                velocity: vel.unwrap_or(100),
                duration: dur,
            },
        ));
        self.do_rest(dur, now);
        self.__coms.send((
            (self.device.clone(), self.channel),
            MidiMsg::StopNote { note },
        ));
    }

    fn set_scale(&mut self, root: String, scale_type: String) {
        let root_midi = note_from_str(root).unwrap_or(60);
        let root = PitchClass::from_midi_note(root_midi);
        let scale_type = match scale_type.to_lowercase().as_str() {
            "maj" | "major" => ScaleType::Major,
            "min" | "minor" => ScaleType::Minor,
            "mel" | "mel-min" => ScaleType::MinorMelodic,
            "harm" | "harm-min" => ScaleType::MinorHarmonic,
            "pent" | "pent-maj" | "maj-pent" => ScaleType::MajorPentatonic,
            "pent-m" | "pentm" | "pent-min" | "min-pent" => ScaleType::MinorPentatonic,
            _ => ScaleType::Major,
        };

        self.scale = Some((Scale::new(root, scale_type), root_midi / 12));
    }

    /// plays a note
    #[pyo3(signature = (note, dur = None, vel = None, blocking = None))]
    fn play(
        &self,
        note: String,
        dur: Option<NoteDuration>,
        vel: Option<u8>,
        blocking: Option<bool>,
    ) {
        self.note(note, dur, vel, blocking);
    }

    pub fn rest(&self, dur: NoteDuration) {
        let now = Instant::now();
        self.do_rest(dur, now);
    }

    #[pyo3(signature = (amt))]
    fn pitch_bend(&self, amt: f32) {
        let y_int_correction = amt + 1.0;
        let bend = (8192. * y_int_correction).floor() as u16;
        // println!(
        //     "bend = {bend}/{}, (from amt: {amt}) on device {:?}:{:?}",
        //     u8::MAX,
        //     self.device,
        //     self.channel
        // );

        self.__coms.send((
            (self.device.clone(), self.channel),
            MidiMsg::PitchBend { bend },
        ));
    }

    #[pyo3(signature = (cc, val))]
    fn cc(&self, cc: u8, val: u8) {
        self.__coms.send((
            (self.device.clone(), self.channel),
            MidiMsg::CC {
                control: cc,
                value: val,
            },
        ));
    }

    #[pyo3(signature = (note))]
    fn stop(&self, note: u8) {
        self.__coms.send((
            (self.device.clone(), self.channel),
            MidiMsg::StopNote { note },
        ));
    }

    #[pyo3(signature = ())]
    fn panic(&self) {
        // TODO: implement this by adding to the enum a panic message
    }
}

#[cfg_attr(feature = "pyo3", pyclass)]
#[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
// #[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Func {
    PyF(Py<PyFunction>),
    PyCF(Py<PyCFunction>),
    PyAny(Py<PyAny>),
}

impl Display for Func {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PyF(_) => write!(f, "PyF"),
            Self::PyCF(_) => write!(f, "PyCF"),
            Self::PyAny(_) => write!(f, "PyAny"),
        }
    }
}

// #[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
// #[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
// #[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
// #[derive(Encode, Decode, Clone)]
#[cfg_attr(feature = "pyo3", pyclass)]
pub struct MidiDaw {
    // NOTE: Only Needed if calling multiple functions
    // riffs: Vec<Py<()>>,
    // riffs: Vec<Py<PyFunction>>,
    threads: FxHashMap<String, Arc<RwLock<thread::MidiDawThread>>>,
    #[pyo3(get, set)]
    pub device: MidiDev,
    // pub device: MidiDeviceName,
    #[pyo3(get, set)]
    pub channel: MidiChannel,
    #[pyo3(get, set)]
    pub block: Option<bool>,
    pub scale: Option<Arc<RwLock<Scale>>>,
    pub tempo: f64,
}

impl MidiDaw {
    // fn mk_decorator
}

#[pymethods]
impl MidiDaw {
    #[new]
    #[pyo3(signature = (dev, channel = MidiChannel::Ch1, tempo = 99.0, virt = false, block = None ))]
    fn new(
        dev: MidiDeviceName,
        channel: MidiChannel,
        tempo: f64,
        virt: bool,
        block: Option<bool>,
    ) -> Self {
        Self {
            // riffs: Vec::new(),
            // threads: Vec::new(),
            threads: FxHashMap::default(),
            device: if virt {
                MidiDev::Virtual(dev)
            } else {
                find_dev(&dev)
                    .map(MidiDev::Physical)
                    .unwrap_or(MidiDev::Physical("Midi Through Port-0".into()))
            },
            channel,
            block,
            scale: None,
            tempo,
        }
    }

    #[pyo3(signature = (func))]
    fn register<'a>(
        &'a mut self,
        py: Python<'a>,
        // func: Py<PyFunction>,
        func: Py<PyAny>,
    ) -> PyResult<Bound<'a, PyCFunction>> {
        let func_name = func.getattr(py, "__name__")?.to_string();
        println!(
            "playing \"{func_name}\" on {:?}:{:?}",
            self.device, self.channel
        );
        // let (tx, rx) = unbounded();
        let api = Api::new(
            self.device.clone(),
            self.channel,
            // tx,
            func_name.to_string(),
            self.tempo,
        );
        // let func = Arc::new(func.bind(py));
        let func_name = Arc::new(func_name);
        // let _jh = Arc::new(Mutex::new(None));
        let block = self.block;
        let exit = Arc::new(AtomicBool::from(false));

        let key = if self.threads.contains_key(&*func_name) {
            let fname = func_name.deref();
            let n = self
                .threads
                .keys()
                .filter(|k| {
                    k.starts_with(fname) && k[(fname.len())..].to_string().parse::<usize>().is_ok()
                })
                .count();

            format!("{}-{n}", fname)
        } else {
            func_name.deref().clone().to_owned()
        };

        let thread = Arc::new(RwLock::new(MidiDawThread::new(
            // func.clone(),
            key.clone(),
            exit.clone(),
            api.clone(),
        )));
        let func = func
            .bind(py)
            .extract::<Py<PyFunction>>()
            .map(Func::PyF)
            .unwrap_or_else(|_| {
                func.bind(py)
                    .extract::<Py<PyCFunction>>()
                    .map(Func::PyCF)
                    .unwrap_or_else(|_| {
                        func.bind(py)
                            .extract::<Py<PyAny>>()
                            .map(Func::PyAny)
                            .unwrap()
                    })
            });
        println!("func, \"{func_name}\", is of type => {func}");
        let func = Arc::new(func);

        println!("storing thread at key: {key}");

        self.threads.insert(key.clone(), thread.clone());

        // let c_f_name = Arc::new(CString::new(func_name.as_bytes().clone().to_vec())?);
        // let c_f_name = c_f_name.clone().as_c_str();

        PyCFunction::new_closure(
            py,
            Some(Box::leak(
                CString::new(
                    func_name
                        .clone()
                        .as_bytes()
                        .to_owned()
                        // .iter()
                        .into_iter()
                        .collect::<Vec<u8>>(),
                )?
                .into_boxed_c_str(),
            )),
            None,
            move |args, kwargs| {
                let func_name = func_name.clone();
                let func = func.clone();
                // let _jh = _jh.clone();
                let mut api = api.clone();
                let thread = thread.clone();
                // let rx = rx.clone();
                let thread_name = key.clone();
                let exit = exit.clone();

                Python::attach(move |py| -> PyResult<String> {
                    let loop_n: Option<usize> = kwargs
                        .map(|kwargs| kwargs.call_method1("pop", ("loops", 1)).ok())
                        .flatten()
                        .map(|loops| loops.extract::<usize>().ok())
                        .flatten();
                    let loc_block: Option<bool> = kwargs
                        .map(|kwargs| kwargs.call_method1("pop", ("block", true)).ok())
                        .flatten()
                        .map(|block| block.extract::<bool>().ok())
                        .flatten();
                    let block = loc_block.unwrap_or_else(|| block.unwrap_or(true));
                    let scale: Option<String> = kwargs
                        .map(|kwargs| kwargs.call_method1("pop", ("scale", true)).ok())
                        .flatten()
                        .map(|scale| scale.extract().ok())
                        .flatten();

                    if let Some(scale) = scale && scale.contains("-") {
                        if let Some((root, scale_type)) = scale.split_once("-") {
                            // println!("root = {root}");
                            api.set_scale(root.into(), scale_type.into());
                        }
                    }
                    exit.store(false, Ordering::Relaxed);

                    // let block = loc_block.unwrap_or_else(|| block.unwrap_or());

                    // println!("loop_n: {loop_n:?}");
                    let loop_n = loop_n.clone().unwrap_or(1);
                    // println!("loop_n: {loop_n:?}");
                    // if loop_n == 0 {
                    //     loop_n = 1;
                    // }
                    // let api = api.into_pyobject(py).unwrap();
                    let loc_api = api.clone().into_pyobject(py).unwrap();
                    let loc_arg = args.to_list();

                    if let Err(e) = loc_arg.insert(0, loc_api) {
                        println!("failed to add api struct to args list...");
                        println!("error message was, \"{e}\"");
                    }

                    let loc_args = loc_arg.to_tuple();

                    let f = {
                        // let func = func.bind(py);
                        // Arc::new(|| func.call(py, (&loc_api,), kwargs))
                        Arc::new(|| {
                            // func.call(py, &loc_args, kwargs)
                            match func.deref() {
                                Func::PyF(func) => func.call(py, &loc_args, kwargs),
                                Func::PyCF(func) => func.call(py, &loc_args, kwargs),
                                Func::PyAny(func) => {
                                    // func.call_method(py, "__call__", &loc_args, kwargs)
                                    func.call(py, &loc_args, kwargs)
                                }
                            }
                        })
                    };
                    let loop_f = || {
                        for _ in 0..loop_n {
                            if let Err(e) = f() {
                                println!("running custom: {func_name}, resulted in error, {e}");
                                break;
                            }
                        }
                    };

                    // if let Err(e) = thread
                    //     .write()
                    //     .map(|mut thread| thread.spawn_midi(api.clone(), rx))
                    // {
                    //     println!("atempt to spawn midi thread failed, :(, with error: {e}");
                    // }

                    if block {
                        // println!("registered, not running in a thread");
                        if loop_n == 0 {
                            // println!("about to loop indefinately");

                            loop {
                                if let Err(e) = f() {
                                    println!("running custom: {func_name}, resulted in error, {e}");
                                    break;
                                }
                            }
                        } else {
                            // println!("about to loop once");

                            loop_f();
                        }
                    } else {
                        // *_jh.lock().unwrap() = Some(if loop_n == 0 {
                        py.detach(|| {
                            if let Err(e) = thread.write().map(|mut thread| {
                                thread.spawn_exec(func, func_name.clone(), loop_n, api.clone())
                            }) {
                                println!("atempt to spawn exec thread: {thread_name} failed, :(, with error: {e}");
                            } else {
                                println!("spawned thread: {thread_name}");
                            }
                        })
                    }

                    Ok(thread_name)
                })
            },
        )
    }

    #[pyo3(signature = (thread_name = None))]
    fn stop(&mut self, thread_name: Option<String>) {
        if let Some(thread_name) = thread_name {
            self.stop_thread(thread_name);
        } else {
            for thread_name in self.threads.clone().keys() {
                self.stop_thread(thread_name.to_owned());
            }
        }
    }

    fn stop_thread(&mut self, thread_name: String) {
        if let Some(thread) = self.threads.get_mut(&thread_name) {
            if let Ok(thread) = thread.write() {
                thread.exit.store(true, Ordering::Relaxed);
                println!("thread, \"{thread_name}\", stop has been triggered.");
            } else {
                println!("failed to write thread exit signal for thread, \"{thread_name}\".");
            }
        } else {
            println!("the thread, \"{thread_name}\", is not registered with this runner.");
        }
    }
}

// #[pyfunction]
// #[pyo3(signature = (func, dev, chan = None, /* loop_n = None, */ tempo = 99.0, block = None))]
// fn play_on<'a>(
//     py: Python<'a>,
//     func: Py<PyFunction>,
//     dev: MidiDeviceName,
//     chan: Option<MidiChannel>,
//     // loop_n: Option<isize>,
//     tempo: Option<f64>,
//     block: Option<bool>,
// ) -> PyResult<Bound<'a, PyCFunction>> {
//     let func_name = func.getattr(py, "__name__")?.to_string();
//     let chan = chan.unwrap_or(MidiChannel::Ch1);
//     println!("playing \"{func_name}\" on {dev}:{chan:?}");
//     let (tx, rx) = unbounded();
//     let physical_dev = find_dev(&dev);
//     let api = Api::new(
//         physical_dev.map(|dev| MidiDev::Physical(dev)).unwrap_or(MidiDev::Virtual(dev)),
//         chan.clone(),
//         tx,
//         func_name.to_string(),
//         tempo.unwrap_or(99.),
//     );
//     let func = Arc::new(func);
//     let func_name = Arc::new(func_name);
//     let _jh = Arc::new(Mutex::new(None));
//
//     PyCFunction::new_closure(py, None, None, move |args, kwargs| {
//         let func_name = func_name.clone();
//         let func = func.clone();
//         let _jh = _jh.clone();
//         let api = api.clone();
//
//         Python::attach(move |py| -> PyResult<()> {
//             let loop_n: Option<usize> = kwargs
//                 .map(|kwargs| kwargs.get_item("loops").ok())
//                 .flatten()
//                 .flatten()
//                 .map(|loops| loops.extract::<usize>().ok())
//                 .flatten();
//             let loc_block: Option<bool> = kwargs
//                 .map(|kwargs| kwargs.get_item("block").ok())
//                 .flatten()
//                 .flatten()
//                 .map(|block| block.extract::<bool>().ok())
//                 .flatten();
//             let block = loc_block.unwrap_or_else(|| block.unwrap_or(true));
//             let loop_n = loop_n.clone().unwrap_or(1);
//             let loc_api = api.clone().into_pyobject(py).unwrap();
//             let f = Arc::new(|| func.call(py, (&loc_api,), kwargs));
//             let loop_f = || {
//                 for _ in 0..loop_n {
//                     if let Err(e) = f() {
//                         println!("running custom: {func_name}, resulted in error, {e}");
//                         break;
//                     }
//                 }
//             };
//
//             if block {
//                 if loop_n == 0 {
//                     loop {
//                         if let Err(e) = f() {
//                             println!("running custom: {func_name}, resulted in error, {e}");
//                             break;
//                         }
//                     }
//                 } else {
//                     loop_f();
//                 }
//             } else {
//                 *_jh.lock().unwrap() = Some(if loop_n == 0 {
//                     let func = func.clone();
//                     let api = api.clone();
//
//                     py.detach(move || {
//                         spawn(move || {
//                             Python::initialize();
//
//                             Python::attach(move |py| {
//                                 let f = {
//                                     let api = api.into_pyobject(py).unwrap();
//
//                                     Arc::new(move || func.call(py, (&api,), kwargs))
//                                 };
//
//                                 loop {
//                                     if let Err(e) = f() {
//                                         println!(
//                                             "running custom: {func_name}, resulted in error, {e}"
//                                         );
//                                         break;
//                                     }
//                                 }
//                             })
//                         })
//                     });
//                 } else {
//                     let func = func.clone();
//                     let api = api.clone();
//
//                     py.detach(move || {
//                         spawn(move || {
//                             Python::initialize();
//
//                             Python::attach(move |py| {
//                                 let loop_f = {
//                                     let api = api.into_pyobject(py).unwrap();
//
//                                     Arc::new(move || {
//                                         for _ in 0..loop_n {
//                                             if let Err(e) = func.call(py, (&api,), kwargs) {
//                                                 println!("running custom: {func_name}, resulted in error, {e}");
//                                                 break;
//                                             }
//                                         }
//                                     })
//                                 };
//
//                                 loop_f()
//                             })
//                         })
//                     });
//                 });
//             }
//
//             Ok(())
//         })
//     })
// }

#[pyfunction]
fn list_devs() -> Vec<String> {
    if let Ok(midi) = MidiOutput::new("midi-daw-search-client") {
        midi.ports()
            .into_iter()
            .filter_map(|p| midi.port_name(&p).ok())
            .collect()
    } else {
        Vec::new()
    }
}

pub fn find_dev(query: &str) -> Option<String> {
    let all_devs = list_devs();
    let devs: Vec<&str> = all_devs.iter().map(|s| s.as_str()).collect();
    let res = fuzzy_search_best_n(query, &devs, 1);

    res.get(0).map(|(dev, _score)| dev.to_string())
}

#[pyfunction]
#[pyo3(name = "find_dev")]
fn py_find_dev(patern: String) -> Option<String> {
    find_dev(&patern)
}

#[cfg_attr(feature = "pyo3", pyclass(from_py_object, unsendable))]
#[derive(Clone)]
pub struct VirtMidiDev(Option<Arc<MidiOutputConnection>>);

#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
// #[cfg_attr(feature = "pyo3", pyclass)]
#[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum MidiDev {
    // Virtual {
    //     name: MidiDeviceName,
    //     // dev: MidiOutputConnection,
    // },
    Virtual(MidiDeviceName),
    Physical(MidiDeviceName),
}

pub fn mk_dev(dev_name: &str) -> Result<MidiOutputConnection, String> {
    match MidiOutput::new("midi-daw") {
        Ok(midi) => match midi.create_virtual(dev_name) {
            Ok(connection) => Ok(connection),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

#[pyfunction]
#[pyo3(name = "mk_dev")]
fn py_mk_dev(dev_name: String) {
    // find_dev(&patern)
    // VirtMidiDev(mk_dev(&dev_name).ok().map(Arc::new))
    _ = mk_dev(&dev_name);
}

// #[pyfunction]
// fn my_decorator(py: Python<'_>, func: Py<PyFunction>) -> PyResult<Bound<'_, PyCFunction>> {
//     // Return a new closure that wraps the original function
//     PyCFunction::new_closure(
//         py,
//         None,
//         None,
//         move |args: &Bound<'_, PyTuple>, kwargs: Option<&Bound<'_, PyDict>>| {
//             Python::attach(|py| -> PyResult<()> {
//                 // Logic before the function call
//                 println!("Before calling the function");
//
//                 // Call the original Python function
//                 // let result = func.call(py, args, kwargs)?;
//                 func.call(py, args, kwargs)?;
//
//                 // Logic after the function call
//                 println!("After calling the function");
//
//                 Ok(())
//             })
//         },
//     )
// }

#[pyfunction]
// #[pyo3(signature = (func))]
fn main<'a>(py: Python<'a>, func: Py<PyAny>) -> PyResult<()> {
    let func = func
        .bind(py)
        .extract::<Py<PyFunction>>()
        .map(Func::PyF)
        .unwrap_or_else(|_| {
            func.bind(py)
                .extract::<Py<PyCFunction>>()
                .map(Func::PyCF)
                .unwrap_or_else(|_| {
                    func.bind(py)
                        .extract::<Py<PyAny>>()
                        .map(Func::PyAny)
                        .unwrap()
                })
        });
    // let builtins = py.import("builtins")?;
    // let var = builtins.getattr("__name__")?;
    // Python::attach(|py| -> PyResult<()> {
    let sys = py.import("sys")?;
    let binder = sys.call_method1("_getframe", (1,));

    if binder.is_err() {
        match func {
            Func::PyF(func) => func.call0(py)?,
            Func::PyCF(func) => func.call0(py)?,
            Func::PyAny(func) => func.call0(py)?,
        };
    }

    Ok(())
    // })
}

#[pyfunction]
#[pyo3(signature = (n = 1))]
fn s4n(n: u8) -> NoteDuration {
    NoteDuration::S4n(n)
}

#[pyfunction]
#[pyo3(signature = (n = 1))]
fn tn(n: u8) -> NoteDuration {
    NoteDuration::Tn(n)
}

#[pyfunction]
#[pyo3(signature = (n = 1))]
fn sn(n: u8) -> NoteDuration {
    NoteDuration::Sn(n)
}

#[pyfunction]
#[pyo3(signature = (n = 1))]
fn en(n: u8) -> NoteDuration {
    NoteDuration::En(n)
}

#[pyfunction]
#[pyo3(signature = (n = 1))]
fn qn(n: u8) -> NoteDuration {
    NoteDuration::Qn(n)
}

#[pyfunction]
#[pyo3(signature = (n = 1))]
fn hn(n: u8) -> NoteDuration {
    NoteDuration::Hn(n)
}

#[pyfunction]
#[pyo3(signature = (n = 1))]
fn wn(n: u8) -> NoteDuration {
    NoteDuration::Wn(n)
}

#[cfg(feature = "pyo3")]
// #[pymodule(gil_used = false)]
#[pymodule]
#[pyo3(submodule, name = "v2")]
/// A Python module implemented in Rust.
pub fn v2(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    let _midi_out_thread: JoinHandle<()> = spawn(midi_out_thread);

    m.add_class::<MidiDaw>()?;
    m.add_function(wrap_pyfunction!(list_devs, m)?)?;
    m.add_function(wrap_pyfunction!(py_find_dev, m)?)?;
    // m.add_function(wrap_pyfunction!(py_mk_dev, m)?)?;
    m.add_function(wrap_pyfunction!(main, m)?)?;

    // note_len
    {
        let module = PyModule::new(py, "note_lens")?;

        module.add_function(wrap_pyfunction!(s4n, m)?)?;
        module.add_function(wrap_pyfunction!(tn, m)?)?;
        module.add_function(wrap_pyfunction!(sn, m)?)?;
        module.add_function(wrap_pyfunction!(en, m)?)?;
        module.add_function(wrap_pyfunction!(qn, m)?)?;
        module.add_function(wrap_pyfunction!(hn, m)?)?;
        module.add_function(wrap_pyfunction!(wn, m)?)?;

        m.add_submodule(&module)?;
        py.import("sys")?
            .getattr("modules")?
            .set_item("midi_daw.v2.note_lens", &module)?;
    }

    // lfo
    {
        let module = PyModule::new(py, "lfo")?;
        automation::lfo_py(py, &module)?;
        m.add_submodule(&module)?;
        py.import("sys")?
            .getattr("modules")?
            .set_item("midi_daw.v2.lfo", &module)?;
    }

    Ok(())
}
