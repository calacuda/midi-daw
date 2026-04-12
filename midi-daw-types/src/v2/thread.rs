use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{JoinHandle, spawn},
};

use pyo3::prelude::*;
use tracing::debug;

use crate::v2::{Api, Func};

// #[cfg_attr(feature = "pyo3", pyclass)]
// #[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
// #[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct MidiDawThread {
    exec_jh: JoinHandle<()>,
    pub exit: Arc<AtomicBool>,
    pub api: Api,
    thread_name: String,
}

impl MidiDawThread {
    pub fn new(thread_name: String, exit: Arc<AtomicBool>, api: Api) -> Self {
        Self {
            exec_jh: spawn(|| {}),
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
        func: Arc<Func>,
        func_name: Arc<String>,
        loop_n: usize,
        mut api: Api,
    ) {
        let exit = self.exit.clone();

        self.exec_jh = if loop_n == 0 {
            debug!(
                "about to spawn loop theads for thread: {}",
                self.thread_name
            );

            spawn(move || {
                Python::initialize();

                Python::attach(move |py| {
                    while exit.load(Ordering::Relaxed) {
                        if let Err(e) = {
                            let api = api.clone().into_pyobject(py).unwrap();

                            match func.deref() {
                                Func::PyF(func) => func.call1(py, (&api,)),
                                Func::PyCF(func) => func.call1(py, (&api,)),
                                Func::PyAny(func) => func.call1(py, (&api,)),
                            }
                        } {
                            debug!("running custom: {func_name}, resulted in error, {e}");
                            api.reset_i();
                            break;
                        } else {
                            api.reset_i();
                        }

                        if exit.load(Ordering::Relaxed) {
                            api.reset_i();
                            break;
                        }
                    }
                })
            })
        } else {
            debug!(
                "about to spawn non-loop theads for thread: {}",
                self.thread_name
            );

            spawn(move || {
                Python::initialize();

                Python::attach(move |py| {
                    for _ in 0..loop_n {
                        if let Err(e) = {
                            let api = api.clone().into_pyobject(py).unwrap();

                            match func.deref() {
                                Func::PyF(func) => func.call1(py, (&api,)),
                                Func::PyCF(func) => func.call1(py, (&api,)),
                                Func::PyAny(func) => func.call1(py, (&api,)),
                            }
                        } {
                            debug!("running custom: {func_name}, resulted in error, {e}");
                            api.reset_i();
                            break;
                        } else {
                            api.reset_i();
                        }

                        if exit.load(Ordering::Relaxed) {
                            api.reset_i();
                            break;
                        }
                    }
                })
            })
        };
    }
}
