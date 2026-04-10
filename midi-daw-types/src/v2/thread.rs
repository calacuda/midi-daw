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

    pub fn is_running(&self) -> bool {
        !self.exec_jh.is_finished()
    }

    pub fn spawn_exec(
        &mut self,
        // func: Arc<Py<PyFunction>>,
        func: Arc<Func>,
        func_name: Arc<String>,
        loop_n: usize,
        mut api: Api,
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
                    // let api = api.into_pyobject(py).unwrap();

                    //     Arc::new(move || func.call1(py, (&api,)))
                    // };

                    while exit.load(Ordering::Relaxed) {
                        // if let Err(e) = f() {
                        // if let Err(e) = func.call1(py, (&api,)) {
                        if let Err(e) = {
                            let api = api.clone().into_pyobject(py).unwrap();

                            match func.deref() {
                                Func::PyF(func) => func.call1(py, (&api,)),
                                Func::PyCF(func) => func.call1(py, (&api,)),
                                Func::PyAny(func) => func.call1(py, (&api,)),
                            }
                        } {
                            println!("running custom: {func_name}, resulted in error, {e}");
                            api.i = 0;
                            break;
                        } else {
                            api.reset_i();
                        }

                        if exit.load(Ordering::Relaxed) {
                            api.i = 0;
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
                    // let api = api.into_pyobject(py).unwrap();

                    // Arc::new(move || {
                    for _ in 0..loop_n {
                        // if let Err(e) = func.call1(py, (&api,)) {
                        if let Err(e) = {
                            let api = api.clone().into_pyobject(py).unwrap();

                            match func.deref() {
                                Func::PyF(func) => func.call1(py, (&api,)),
                                Func::PyCF(func) => func.call1(py, (&api,)),
                                Func::PyAny(func) => func.call1(py, (&api,)),
                            }
                        } {
                            println!("running custom: {func_name}, resulted in error, {e}");
                            api.i = 0;
                            break;
                        } else {
                            // api.increment();
                            api.reset_i();
                        }

                        if exit.load(Ordering::Relaxed) {
                            api.i = 0;
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
