use crate::{MidiChannel, MidiDeviceName, NoteDuration};
use bincode::{Decode, Encode};
use crossbeam::channel::{Sender, unbounded};
#[cfg(feature = "pyo3")]
use log::*;
use midi_msg::Channel;
use pyo3::types::{PyCFunction, PyDict, PyTuple};
#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, types::PyFunction};
use serde::{Deserialize, Serialize};
use std::{
    ops::Deref,
    sync::{Arc, Mutex},
    thread::{JoinHandle, spawn},
    time::Duration,
};
#[cfg(not(feature = "pyo3"))]
use tracing::*;

// #[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
// #[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
// #[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Clone)]
pub struct Api {
    // NOTE: Only Needed if calling multiple functions
    // riffs: Vec<Py<()>>,
    #[pyo3(get, set)]
    pub device: MidiDeviceName,
    #[pyo3(get, set)]
    pub channel: MidiChannel,
    // __threads: Vec<JoinHandle<()>>,
    __coms: Sender<()>,
    __name: String,
}

impl Api {
    fn new(dev: MidiDeviceName, channel: MidiChannel, __coms: Sender<()>, name: String) -> Self {
        Self {
            device: dev,
            channel,
            // __threads: Vec::new(),
            __coms,
            __name: name,
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
    fn note(
        &self,
        note: String,
        dur: Option<NoteDuration>,
        vel: Option<i8>,
        blocking: Option<bool>,
    ) {
    }
}

// #[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
// #[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
// #[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
// pub struct Func {}

#[cfg_attr(feature = "pyo3", pyclass)]
// #[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
// #[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct MidiDaw {
    // NOTE: Only Needed if calling multiple functions
    // riffs: Vec<Py<()>>,
    riffs: Vec<Py<PyFunction>>,
    threads: Vec<JoinHandle<()>>,
    pub device: MidiDeviceName,
    pub channel: MidiChannel,
}

impl MidiDaw {
    // fn mk_decorator
}

#[pymethods]
impl MidiDaw {
    #[new]
    fn new(dev: MidiDeviceName, channel: MidiChannel) -> Self {
        Self {
            riffs: Vec::new(),
            threads: Vec::new(),
            device: dev,
            channel,
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
}

#[pyfunction]
#[pyo3(signature = (func, dev, chan = None, /* loop_n = None, */ block = None))]
fn my_decorator_factory(
    py: Python<'_>,
    func: Py<PyFunction>,
    dev: MidiDeviceName,
    chan: Option<MidiChannel>,
    // loop_n: Option<isize>,
    block: Option<bool>,
) -> PyResult<Bound<'_, PyCFunction>> {
    println!("after func");
    let func_name = func.getattr(py, "__name__")?.to_string();
    println!("func-name: {func_name}");
    let (tx, rx) = unbounded();
    let api = Api::new(
        dev.clone(),
        chan.unwrap_or(MidiChannel::Ch1),
        tx,
        func_name.to_string(),
    );
    // let mut loop_n = loop_n.clone().map(|l| l.abs() as usize).unwrap_or(1);
    // if loop_n == 0 {
    //     loop_n = 1;
    // }

    // let blocking = block.unwrap_or(true);
    // let func = Arc::new(func.unbind());
    let func = Arc::new(func);
    let func_name = Arc::new(func_name);
    // let func = Arc::new(func.bind(py));
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
                        println!("running custom: {func_name}, rsulted in error, {e}");
                        break;
                    }
                }
            };

            if block {
                if loop_n == 0 {
                    loop {
                        if let Err(e) = f() {
                            println!("running custom: {func_name}, rsulted in error, {e}");
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
    m.add_function(wrap_pyfunction!(my_decorator_factory, m)?)?;

    Ok(())
}
