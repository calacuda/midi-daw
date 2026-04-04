use crate::{MidiChannel, MidiDeviceName, MidiMsg, NoteDuration, v1::note_from_str, v2::thread::MidiDawThread};
use bincode::{Decode, Encode};
use crossbeam::channel::{Sender, unbounded};
#[cfg(feature = "pyo3")]
use log::*;
use midir::{ConnectError, MidiInput, MidiOutput, MidiOutputConnection, os::unix::VirtualOutput};
use pyo3::types::PyCFunction;
#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, types::PyFunction};
use rust_fuzzy_search::fuzzy_search_best_n;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::{
    ops::Deref,
    sync::{Arc, Mutex, RwLock, atomic::AtomicBool},
    thread::{sleep, spawn}, time::Duration,
};
#[cfg(not(feature = "pyo3"))]
use tracing::*;

// pub mod v2;
pub mod thread;

pub type Scale = Vec<String>;
pub type MidiThreadCtrlMesg = MidiMsg;

// #[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
// #[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
// #[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Clone, Debug)]
pub struct Api {
    // NOTE: Only Needed if calling multiple functions
    // riffs: Vec<Py<()>>,
    #[pyo3(get, set)]
    pub device: MidiDev,
    #[pyo3(get, set)]
    pub channel: MidiChannel,
    // __threads: Vec<JoinHandle<()>>,
    __coms: Sender<MidiThreadCtrlMesg>,
    __name: String,
    tempo: f64,
}

impl Api {
    fn new(dev: MidiDev, channel: MidiChannel, __coms: Sender<MidiThreadCtrlMesg>, name: String, tempo: f64) -> Self {
        Self {
            device: dev,
            channel,
            // __threads: Vec::new(),
            __coms,
            __name: name,
            tempo,
        }
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

    /// starts playback
    fn start(&self) {}

    /// plays a note
    #[pyo3(signature = (note, dur = None, vel = None, blocking = None))]
    fn note(
        &self,
        note: String,
        dur: Option<NoteDuration>,
        vel: Option<u8>,
        blocking: Option<bool>,
    ) {
        let dur = dur.unwrap_or(NoteDuration::Sn(1));
        println!("playing {note}@{vel:?} for {dur:?}, on: {:?}. blocking? {blocking:?}", self.device);
        let note = note_from_str(note).unwrap_or(0);
        self.__coms.send(
            MidiThreadCtrlMesg::PlayNote { 
                note, 
                velocity: vel.unwrap_or(100), 
                duration: dur,
            });
        self.rest(dur);
        self.__coms.send(MidiThreadCtrlMesg::StopNote { note });
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

        sleep(Duration::from_secs_f64(
            // ((60.0 / self.tempo) * 2.0 / denom) * mul,
            ((60.0 / self.tempo) * denom) * mul,
        ));
    }
}

// #[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
// #[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
// #[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
// pub struct Func {}

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
    fn new(dev: MidiDeviceName, channel: MidiChannel, tempo: f64, virt: bool, block: Option<bool>) -> Self {
        Self {
            // riffs: Vec::new(),
            // threads: Vec::new(),
            threads: FxHashMap::default(),
            device: if virt { 
                MidiDev::Virtual(dev)
            } else {
                find_dev(&dev)
                    .map(|dev| MidiDev::Physical(dev))
                    .unwrap_or(MidiDev::Physical("Midi Through Port-0".into()))
            },
            channel,
            block,
            scale: None,
            tempo,
        }
    }

    // fn register<'a>(
    //     &mut self,
    //     py: Python<'a>,
    //     func: Py<PyFunction>,
    // ) -> PyResult<Bound<'a, PyCFunction>> {
    //     // Return a new closure that wraps the original function
    //     PyCFunction::new_closure(
    //         py,
    //         None,
    //         None,
    //         move |args: &Bound<'_, PyTuple>, kwargs: Option<&Bound<'_, PyDict>>| {
    //             Python::attach(|py| -> PyResult<Py<PyAny>> {
    //                 // Python::attach(|py| {
    //                 // Logic before the function call
    //                 println!("Before calling the function");
    //
    //                 let func_name = func.getattr(py, "__name__").ok();
    //
    //                 if func_name.is_none() {
    //                     println!("functions name parameter was missing. why is that?");
    //                 }
    //
    //                 // Call the original Python function
    //                 // let result = func.call(py, args, kwargs)?;
    //                 let result = func.call(py, args, kwargs);
    //
    //                 // if let Err(e) = func.call(py, args, kwargs) {
    //                 match result {
    //                     Ok(res) => {
    //                         // Logic after the function call
    //                         println!("After calling the function");
    //
    //                         Ok(res)
    //                     }
    //                     Err(e) => {
    //                         println!(
    //                             "{} produced error: {e}",
    //                             func_name
    //                                 .map(|func_name| format!("wrapped fucntion: \"{func_name}\""))
    //                                 .unwrap_or("python fucntion".into())
    //                         );
    //
    //                         Err(e)
    //                     }
    //                 }
    //             })
    //         },
    //     )
    // }

    //     fn register<'a>(
    //         &mut self,
    //         // py: Python<'_>,
    //         // func: Py<PyFunction>,
    //         loop_n: Option<isize>,
    //         block: Option<bool>,
    //     ) -> PyResult<Bound<'_, PyCFunction>> {
    //         // // let mk_decorator = move |func_name: Option<Py<PyAny>>, func: Py<PyFunction>| {
    //         // let mk_decorator = move |args: Py<PyTuple>| {
    //         //     // let func = args.extract::<(Py<PyFunction>,)>()?.0; // The function being decorated
    //         //     let func = args.into_bound(py).extract::<(Py<PyFunction>,)>()?.0; // The function being decorated
    //         //     let func_name = func.getattr(py, "__name__").ok();
    //         //
    //         //     if func_name.is_none() {
    //         //         println!("functions name parameter was missing. why is that?");
    //         //     }
    //         //
    //         //     self.riffs.push(func);
    //         //
    //         //     // This is the wrapper that replaces the original function
    //         //     // Return a new closure that wraps the original function
    //         //     PyCFunction::new_closure(
    //         //         py,
    //         //         func_name.map(|f_name| {
    //         //             std::ffi::CString::new(f_name.to_string().into_bytes())
    //         //                 .as_deref()
    //         //                 .unwrap()
    //         //         }),
    //         //         None,
    //         //         move |inner_args: &Bound<'_, PyTuple>, inner_kwargs: Option<&Bound<'_, PyDict>>| {
    //         //             Python::attach(|py| -> PyResult<Py<PyAny>> {
    //         //                 // Python::attach(|py| -> PyResult<Bound<'_, PyAny>> {
    //         //                 println!("Before calling the function");
    //         //
    //         //                 if func_name.is_none() {
    //         //                     println!("functions name parameter was missing. why is that?");
    //         //                 }
    //         //
    //         //                 // Call the original Python function
    //         //                 let result = func.call(py, inner_args, inner_kwargs);
    //         //
    //         //                 match result {
    //         //                     Ok(res) => {
    //         //                         println!("After calling the function");
    //         //
    //         //                         Ok(res)
    //         //                     }
    //         //                     Err(e) => {
    //         //           pyo3 0.28 with out gil              println!(
    //         //                             "{} produced error: {e}",
    //         //                             func_name
    //         //                                 .map(|func_name| format!(
    //         //                                     "wrapped fucntion: \"{func_name}\""
    //         //                                 ))
    //         //                                 .unwrap_or("python fucntion".into())
    //         //                         );
    //         //
    //         //                         Err(e)
    //         //                     }
    //         //                 }
    //         //             })
    //         //         },
    //         //     )
    //         // };
    //
    //         Python::attach(|py| {
    //             // This is the actual decorator that Python will receive
    //             PyCFunction::new_closure(
    //                 py,
    //                 Some(c"play_on_generator"),
    //                 None,
    //                 move |args, _kwargs| -> PyResult<Py<Api>> {
    //                     // let func = args.extract::<(Py<PyFunction>,)>()?.0; // The function being decorated
    //                     // let func: Bound<'_, PyAny> = args.get_item(0)?; // The function being decorated
    //                     // func.unbind().bind(py);
    //                     // let loc_py = func.py().clone();
    //                     // let func = func.unbind();
    //
    //                     let loop_n = loop_n.clone().unwrap_or(0);
    //                     let block = block.unwrap_or(true);
    //
    //                     let func = args.extract::<(Py<PyFunction>,)>()?.0; // The function being decorated
    //                     // let func_name = func.getattr(py, "__name__").ok();
    //                     //
    //                     // if func_name.is_none() {
    //                     //     println!("functions name parameter was missing. why is that?");
    //                     // }
    //
    //                     let func_name = func.getattr(py, "__name__")?;
    //                     // let func_name = func.getattr("__name__")?;
    //
    //                     let (tx, rx) = unbounded();
    //                     let api = Api::new(self.device, self.channel, tx, func_name.to_string())
    //                         .into_pyobject(py)
    //                         .unwrap()
    //                         .unbind();
    //
    //                     self.riffs.push(func);
    //                     self.threads.push(spawn(move || {
    //                         // Python::attach(|py| -> PyResult<Bound<'_, PyAny>> {
    //                         println!("Before calling the function");
    //
    //                         // if func_name.is_none() {
    //                         //     println!("functions name parameter was missing. why is that?");
    //                         // }
    //
    //                         // Call the original Python function
    //                         let result = Python::attach(|py| func.call1(py, (api,)));
    //                         // let result = func.call1((api,));
    //
    //                         match result {
    //                             Ok(res) => {
    //                                 println!("After calling the function");
    //
    //                                 // Ok(res)
    //                             }
    //                             Err(e) => {
    //                                 println!("wrapped fucntion: \"{func_name}\" produced error: {e}");
    //
    //                                 // Err(e)
    //                             }
    //                         }
    //                     }));
    //
    //                     Ok(api)
    //
    //                     // mk_decorator(func_name, func)
    //                     // mk_decorator(func)
    //                     // mk_decorator(args.clone().unbind())
    //                 },
    //             )
    //         })
    //     }
    // }

    #[pyo3(signature = (func))]
    fn register<'a>(
        &'a mut self,
        py: Python<'a>,
        func: Py<PyFunction>,
    ) -> PyResult<Bound<'a, PyCFunction>> {
        let func_name = func.getattr(py, "__name__")?.to_string();
        println!("playing \"{func_name}\" on {:?}:{:?}", self.device, self.channel);
        let (tx, rx) = unbounded();
        let api = Api::new(
            self.device.clone(),
            self.channel.clone(),
            tx,
            func_name.to_string(),
            self.tempo,
        );
        let func = Arc::new(func);
        let func_name = Arc::new(func_name);
        // let _jh = Arc::new(Mutex::new(None));
        let block = self.block;
        let exit = Arc::new(AtomicBool::from(false));
        let thread = Arc::new(RwLock::new(MidiDawThread::new(func.clone(), exit.clone(), api.clone())));
        
        let key = if self.threads.contains_key(&*func_name) {
            let fname = func_name.deref();
            let n = self.threads.keys().filter(|k| k.starts_with(fname) && k[(fname.len())..].to_string().parse::<usize>().is_ok()).count();

            format!("{}-{n}", fname)
        } else {
            func_name.deref().clone()
        };

        self.threads.insert(key.clone(), thread.clone());

        PyCFunction::new_closure(py, None, None, move |args, kwargs| {
            let func_name = func_name.clone();
            let func = func.clone();
            // let _jh = _jh.clone();
            let api = api.clone();
            let thread = thread.clone();
            let rx = rx.clone();
            let thread_name = key.clone();

            Python::attach(move |py| -> PyResult<String> {
                let loop_n: Option<usize> = kwargs
                    .map(|kwargs| kwargs.get_item("loops").ok())
                    .flatten()
                    .flatten()
                    .map(|loops| loops.extract::<usize>().ok())
                    .flatten();
                let loc_block: Option<bool> = kwargs
                    .map(|kwargs| kwargs.get_item("block").ok())
                    .flatten()
                    .flatten()
                    .map(|block| block.extract::<bool>().ok())
                    .flatten();
                let block = loc_block.unwrap_or_else(|| block.unwrap_or(true));

                // println!("loop_n: {loop_n:?}");
                let loop_n = loop_n.clone().unwrap_or(1);
                // println!("loop_n: {loop_n:?}");
                // if loop_n == 0 {
                //     loop_n = 1;
                // }
                // let api = api.into_pyobject(py).unwrap();
                let loc_api = api.clone().into_pyobject(py).unwrap();
                let f = {
                    // let func = func.bind(py);
                    Arc::new(|| func.call1(py, (&loc_api,)))
                };
                let loop_f = || {
                    for _ in 0..loop_n {
                        if let Err(e) = f() {
                            println!("running custom: {func_name}, resulted in error, {e}");
                            break;
                        }
                    }
                };
                
                if let Err(e) = thread.write().map(|mut thread| thread.spawn_midi(api.clone(), rx)) {
                    println!("atempt to spawn midi thread failed, :(, with error: {e}");
                }

                if block {
                    if loop_n == 0 {
                        loop {
                            if let Err(e) = f() {
                                println!("running custom: {func_name}, resulted in error, {e}");
                                break;
                            }
                        }
                    } else {
                        loop_f();
                    }
                } else {
                    // *_jh.lock().unwrap() = Some(if loop_n == 0 {
                    py.detach(move || {
                        if let Err(e) = thread.write().map(|mut thread| thread.spawn_exec(func, func_name.clone(), loop_n, api.clone())) {
                            println!("atempt to spawn exec thread failed, :(, with error: {e}");
                        }
                    })
                }

                Ok(thread_name)
            })
        })
    }
}

#[pyfunction]
#[pyo3(signature = (func, dev, chan = None, /* loop_n = None, */ tempo = 99.0, block = None))]
fn play_on<'a>(
    py: Python<'a>,
    func: Py<PyFunction>,
    dev: MidiDeviceName,
    chan: Option<MidiChannel>,
    // loop_n: Option<isize>,
    tempo: Option<f64>,
    block: Option<bool>,
) -> PyResult<Bound<'a, PyCFunction>> {
    let func_name = func.getattr(py, "__name__")?.to_string();
    let chan = chan.unwrap_or(MidiChannel::Ch1);
    println!("playing \"{func_name}\" on {dev}:{chan:?}");
    let (tx, rx) = unbounded();
    let physical_dev = find_dev(&dev);
    let api = Api::new(
        physical_dev.map(|dev| MidiDev::Physical(dev)).unwrap_or(MidiDev::Virtual(dev)),
        chan.clone(),
        tx,
        func_name.to_string(),
        tempo.unwrap_or(99.),
    );
    let func = Arc::new(func);
    let func_name = Arc::new(func_name);
    let _jh = Arc::new(Mutex::new(None));

    PyCFunction::new_closure(py, None, None, move |args, kwargs| {
        let func_name = func_name.clone();
        let func = func.clone();
        let _jh = _jh.clone();
        let api = api.clone();

        Python::attach(move |py| -> PyResult<()> {
            let loop_n: Option<usize> = kwargs
                .map(|kwargs| kwargs.get_item("loops").ok())
                .flatten()
                .flatten()
                .map(|loops| loops.extract::<usize>().ok())
                .flatten();
            let loc_block: Option<bool> = kwargs
                .map(|kwargs| kwargs.get_item("block").ok())
                .flatten()
                .flatten()
                .map(|block| block.extract::<bool>().ok())
                .flatten();
            let block = loc_block.unwrap_or_else(|| block.unwrap_or(true));
            let loop_n = loop_n.clone().unwrap_or(1);
            let loc_api = api.clone().into_pyobject(py).unwrap();
            let f = Arc::new(|| func.call1(py, (&loc_api,)));
            let loop_f = || {
                for _ in 0..loop_n {
                    if let Err(e) = f() {
                        println!("running custom: {func_name}, resulted in error, {e}");
                        break;
                    }
                }
            };

            if block {
                if loop_n == 0 {
                    loop {
                        if let Err(e) = f() {
                            println!("running custom: {func_name}, resulted in error, {e}");
                            break;
                        }
                    }
                } else {
                    loop_f();
                }
            } else {
                *_jh.lock().unwrap() = Some(if loop_n == 0 {
                    let func = func.clone();
                    let api = api.clone();

                    py.detach(move || {
                        spawn(move || {
                            Python::initialize();

                            Python::attach(move |py| {
                                let f = {
                                    let api = api.into_pyobject(py).unwrap();

                                    Arc::new(move || func.call1(py, (&api,)))
                                };

                                loop {
                                    if let Err(e) = f() {
                                        println!(
                                            "running custom: {func_name}, resulted in error, {e}"
                                        );
                                        break;
                                    }
                                }
                            })
                        })
                    });
                } else {
                    let func = func.clone();
                    let api = api.clone();

                    py.detach(move || {
                        spawn(move || {
                            Python::initialize();

                            Python::attach(move |py| {
                                let loop_f = {
                                    let api = api.into_pyobject(py).unwrap();
                                    
                                    Arc::new(move || {
                                        for _ in 0..loop_n {
                                            if let Err(e) = func.call1(py, (&api,)) {
                                                println!("running custom: {func_name}, resulted in error, {e}");
                                                break;
                                            }
                                        }
                                    })
                                };

                                loop_f()
                            })
                        })
                    });
                });
            }

            Ok(())
        })
    })
}

#[pyfunction]
fn list_devs() -> Vec<String> {
    if let Ok(midi) = MidiOutput::new("midi-daw-search-client") {
        midi.ports().into_iter().filter_map(|p| {
            midi.port_name(&p).ok()
        }).collect()
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
#[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum MidiDev {
    Virtual(MidiDeviceName),
    Physical(MidiDeviceName),
}

pub fn mk_dev(dev_name: &str) -> Result<MidiOutputConnection, String> {
    match 
    MidiOutput::new("midi-daw") {
        Ok(midi) => match midi.create_virtual(dev_name) {
            Ok(connection) => Ok(connection),
            Err(e) => Err(e.to_string())
        }
        Err(e) => Err(e.to_string())
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

#[cfg(feature = "pyo3")]
#[pymodule]
#[pyo3(submodule, name = "v2")]
/// A Python module implemented in Rust.
pub fn v2(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<MidiDaw>()?;
    // m.add_function(wrap_pyfunction!(my_decorator, m)?)?;
    // m.add_function(wrap_pyfunction!(my_decorator_factory, m)?)?;
    m.add_function(wrap_pyfunction!(play_on, m)?)?;
    m.add_function(wrap_pyfunction!(list_devs, m)?)?;
    m.add_function(wrap_pyfunction!(py_find_dev, m)?)?;
    m.add_function(wrap_pyfunction!(py_mk_dev, m)?)?;

    Ok(())
}
