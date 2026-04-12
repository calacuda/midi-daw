use std::{
    ffi::CString, ops::Deref, sync::{Arc, Mutex}, thread::{sleep, spawn}, time::{Duration}
};

use bincode::{Decode, Encode};
use enum_dispatch::enum_dispatch;
use hound::{SampleFormat, WavReader};
use lfo::{Lfo};
use pyo3::types::{PyCFunction, PyFunction};
#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, types::PyDict};
use serde::{Deserialize, Serialize};

use crate::{
    v2::{
        automation::lfo::{sin_wave::SinLfo, wavetable::WaveTable, LfoConfig},
    },
};

// pub mod envelope;
pub mod lfo;

pub static AUTOMATIONS_PER_SECOND: f64 = 48_000.0;

#[enum_dispatch]
pub trait AutomationTraitV2 /*: PyClass */ {
    // fn automation_type(&self) -> impl Into<String>;
    fn sub_type(&self) -> String;
    /// used to update the state of the automation
    fn update(&mut self);
    /// used to get the last value of automation
    fn get_value(&self) -> f64;
    /// checks if the lfo has finished and should stop processing
    fn done(&self) -> bool;
    fn step(&mut self) -> f64 {
        self.update();
        self.get_value()
    }
}

// #[pyclass]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
#[enum_dispatch(AutomationTraitV2)]
pub enum AutomationTypes {
    Lfo(lfo::Lfo),
    // EnvelopeGen(envelope::Envelope),
}

// #[pyclass]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, PartialOrd, Clone, Debug)]
pub enum AutomationConf {
    Lfo(lfo::LfoConfig),
    // EnvelopeGen(envelope::EnvConfig),
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl AutomationConf {
    // This method enables Python's copy.deepcopy()
    pub fn __deepcopy__(&self, _memo: &Bound<'_, PyDict>) -> Self {
        self.clone() // Use Rust's Clone implementation
    }
}

impl TryFrom<AutomationConf> for AutomationTypes {
    type Error = String;

    fn try_from(value: AutomationConf) -> Result<Self, Self::Error> {
        match value {
            AutomationConf::Lfo(lfo::LfoConfig::WaveTable { file, freq }) => {
                // read wav file to Vec<f64>
                let mut reader = WavReader::open(file)
                    .map_err(|e| format!("failed to read wav file. tried and got error: {e}"))?;
                // let samples = reader
                //     .samples::<i32>()
                //     .map(|sample| {
                //         let sample = sample
                //             .map_err(|e| format!("sample failed to decode with error: {e}"))?;
                //         Ok((sample as f64) / (i32::MAX as f64))
                //     })
                //     .collect::<Result<Vec<f64>, String>>()?;
                let samples = match reader.spec().sample_format {
                    SampleFormat::Int => reader
                        .samples::<i32>()
                        .map(|sample| {
                            let sample = sample
                                .map_err(|e| format!("sample failed to decode with error: {e}"))?;
                            Ok((sample as f64) / (i32::MAX as f64))
                        })
                        .collect::<Result<Vec<f64>, String>>()?,

                    SampleFormat::Float => reader
                        .samples::<f32>()
                        .map(|sample| {
                            let sample = sample
                                .map_err(|e| format!("sample failed to decode with error: {e}"))?;
                            // Ok((sample as f64) / (i32::MAX as f64))
                            Ok(sample as f64)
                        })
                        .collect::<Result<Vec<f64>, String>>()?,
                };

                // build WaveTable
                let mut wavetable =
                    WaveTable::new(samples.into(), reader.spec().sample_rate as f64);
                wavetable.set_frequency(freq);

                // set WaveTable frequency to freq
                Ok(AutomationTypes::Lfo(lfo::Lfo::WaveTable(wavetable)))
            }
            AutomationConf::Lfo(lfo::LfoConfig::Sin {
                freq,
                one_shot,
                bipolar,
                hifi,
                sample_rate,
            }) => Ok(AutomationTypes::Lfo(lfo::Lfo::Sin(SinLfo::new(
                freq, one_shot, bipolar, hifi, sample_rate,
            )))), // AutomationConf::EnvelopeGen() => {}
        }
    }
}

// #[pyclass]
#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Debug)]
pub struct Automation {
    automation: AutomationTypes,
    out_port: jack::Port<jack::AudioOut>,
    // pub jack_client: ,
}

// #[pymethods]
// #[cfg_attr(feature = "pyo3", pymethods)]
impl Automation {
    // #[cfg(feature = "pyo3")]
    // #[new]
    // pub fn new(conf: AutomationConf) -> PyResult<Self> {
    //     let automation = match AutomationTypes::try_from(conf) {
    //         Ok(automation) => automation,
    //         Err(e) => {
    //             eprintln!("making automation failed with error: {e}");
    //             return Err(PyErr::new::<PyValueError, _>(e.to_string()));
    //         }
    //     };
    //
    //     Ok(Self { automation })
    // }
    //
    // #[cfg(not(feature = "pyo3"))]
    pub fn new(conf: AutomationConf, name: &str) -> Result<(Self, jack::Client), String> {
        let automation = match AutomationTypes::try_from(conf) {
            Ok(automation) => automation,
            Err(e) => {
                // eprintln!("making automation failed with error: {e}");
                return Err(e.to_string());
            }
        };

        // 1. Open a client
        let (client, _status) =
            jack::Client::new(&"midi-daw", jack::ClientOptions::default()).unwrap();

        // 2. Register port
        let out_port = client
            .register_port(&name, jack::AudioOut::default())
            .unwrap();

        Ok((
            Self {
                automation,
                out_port,
                // jack_client: client,
            },
            client,
        ))
    }

    pub fn step(&mut self) -> f64 {
        self.automation.step()
    }

    pub fn get_repr(&self) -> String {
        match self.automation.clone() {
            AutomationTypes::Lfo(lfo) => format!("lfo:{}", lfo.sub_type()),
            // AutomationTypes::EnvelopeGen(env) => format!("env:{}", env.sub_type()),
        }
    }

    pub fn done(&self) -> bool {
        self.automation.done()
    }

    // fn automation_type(&self) -> String {
    // self.automation_step().into()
    // }

    pub fn sub_type(&self) -> String {
        self.automation.sub_type().into()
    }
}

pub enum AutomationNotification {
    Done,
}

impl jack::contrib::controller::ControlledProcessorTrait for Automation {
    type Command = ();
    type Notification = AutomationNotification;

    fn buffer_size(
        &mut self,
        _client: &jack::Client,
        _size: jack::Frames,
        _channels: &mut jack::contrib::controller::ProcessorChannels<
            Self::Command,
            Self::Notification,
        >,
    ) -> jack::Control {
        jack::Control::Continue
    }

    fn process(
        &mut self,
        _client: &jack::Client,
        scope: &jack::ProcessScope,
        channels: &mut jack::contrib::controller::ProcessorChannels<
            Self::Command,
            Self::Notification,
        >,
    ) -> jack::Control {
        let name = self.out_port.name().unwrap();
        let out = self.out_port.as_mut_slice(scope);

        for sample in out.iter_mut() {
            // let x = self.frequency * self.time * 2.0 * PI;
            // *sample = (x.sin() as f32) * gain;
            // self.time += self.frame_t;
            *sample = self.automation.step() as f32;
            // info!("{sample}");

            if self.automation.done() {
                if let Err(_e) = channels.try_notify(AutomationNotification::Done) {
                    eprintln!("failed to notify controller that {}, is done...", name);
                }
            }
        }

        jack::Control::Continue
    }
}

#[pyfunction]
#[pyo3(signature = (func, /* dev, chan = None, */ freq = 6.0, bi_pole = false, hifi = false, /* tempo = 99.0, */ ))]
fn sin<'a>(
    py: Python<'a>,
    func: Py<PyFunction>,
    // dev: MidiDeviceName,
    // chan: Option<MidiChannel>,
    freq: f32,
    bi_pole: bool,
    hifi: bool,
    // tempo: Option<f64>,
) -> PyResult<Bound<'a, PyCFunction>> {
    let func_name = func.getattr(py, "__name__")?.to_string();
    // let chan = chan.unwrap_or(MidiChannel::Ch1);
    // println!("making sin wave lfo, \"{func_name}\", on {dev}:{chan:?}");
    // let (tx, rx) = unbounded();
    // let physical_dev = find_dev(&dev);
    // let api = Api::new(
    //     physical_dev
    //         .map(|dev| MidiDev::Physical(dev))
    //         .unwrap_or(MidiDev::Virtual(dev)),
    //     chan.clone(),
    //     tx,
    //     func_name.to_string(),
    //     tempo.unwrap_or(99.),
    // );
    let func = Arc::new(func);
    let func_name = Arc::new(func_name);
    let _jh = Arc::new(Mutex::new(None));

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
            let _jh = _jh.clone();
            // let api = api.clone();
            let args = Arc::new(args.clone().unbind());
            let freq: f32 = kwargs
                // .map(|kwargs| kwargs.get_item("freq").ok())
                // .map(|kwargs| kwargs.call_method1("pop", ("freq", 6.0)).ok())
                .map(|kwargs| kwargs.call_method1("pop", ("freq", freq)).ok())
                .flatten()
                // .flatten()
                .map(|freq| freq.extract::<f32>().ok())
                .flatten()
                .unwrap_or(6.0);
            let kwargs = Arc::new(kwargs.map(|kwargs| kwargs.clone().unbind()));
            let func = func.clone();

            Python::attach(move |py| -> PyResult<()> {
                // let api: Api = kwargs
                //     .map(|kwargs| kwargs.get_item("api").ok())
                //     .flatten()
                //     .flatten()
                //     .map(|freq| freq.extract::<f32>().ok())
                //     .flatten()
                //     .unwrap_or(6.0);
                // let loc_api = api.clone().into_pyobject(py).unwrap();
                // let f = Arc::new(|| func.call1(py, (&loc_api,)));

                // let mut auto = AutomationTypes::try_from(AutomationConf::Lfo(LfoConfig::Sin {
                //     freq: freq as f64,
                //     one_shot: false,
                //     bipolar: bi_pole,
                //     hifi,
                // }));

                let sample_rate = if hifi { 44_100. } else { 22_050. };
                // let sample_rate = if hifi { 11_025. } else { 751.5625 };
                // println!("sample-rate = {sample_rate}");
                let wait_time = Duration::from_secs_f32(1. / sample_rate);
                // println!("wait-time: {wait_time:?}");
                // let kwargs = kwargs.map(|kwargs| kwargs.bind(py));
                // let kwargs = kwargs.map(|kwargs| kwargs.bind(py).clone().unbind());
                // let args = args.bind(py).clone().unbind();
                let args = args.clone();
                let kwargs = kwargs.clone();

                *_jh.lock().unwrap() = Some({
                    // let func = func.clone();
                    // let api = api.clone();
                    // let auto = auto.clone();
                    let auto = AutomationTypes::try_from(AutomationConf::Lfo(LfoConfig::Sin {
                        freq: freq as f64,
                        one_shot: false,
                        bipolar: bi_pole,
                        sample_rate: sample_rate as f64,
                        hifi,
                    }));
                    // let kwargs = kwargs.map(|kwargs| kwargs.unbind());
                    // let kwargs = kwargs.clone().map(|kwargs| kwargs.unbind());
                    // let kwargs = kwargs.map(|kwargs| kwargs.bind(py).clone().unbind());
                    let func = func.clone();
                    // let args = args.bind(py).clone().unbind();
                    let args = args.clone();
                    let kwargs = kwargs.clone();

                    py.detach(move || {
                        spawn(move || {
                            Python::initialize();

                            // Python::attach(move |py| {
                                if let Ok(mut auto) = auto {
                                    // let f = {
                                    //     // let api = api.into_pyobject(py).unwrap();
                                    //     // let kwargs = match kwargs {
                                    //     //     Some(kwargs) => Some(kwargs.bind(py)),
                                    //     //     None => None,
                                    //     // };
                                    //     // let kwargs = kwargs.map(|kwargs| kwargs.bind(py).to_owned());
                                    //     // let kwargs = kwargs.clone();
                                    //
                                    //     // let kwargs = kwargs.as_ref();
                                    //
                                    //     // Arc::new(move |s: f32| func.call(py, (&api, s), kwargs))
                                    //     move |py, kwargs: Option<Py<PyDict>>, s| {
                                    //         let kwargs = kwargs.map(|kwargs| kwargs.bind(py).to_owned());
                                    //         let args = args.bind(py).to_list();
                                    //
                                    //         if let Err(e) = args.append(s as f32) {
                                    //             println!("couldn't add sample to args list. {e}");
                                    //         }
                                    //
                                    //         let args = args.to_tuple();
                                    //
                                    //         // func.call(py, args, kwargs)
                                    //         func.call(
                                    //             py,
                                    //             args,
                                    //             kwargs.as_ref(), // .map(|kwargs| kwargs.bind(py).to_owned())
                                    //                             // .as_ref(),
                                    //         )
                                    //         // Ok(())
                                    //     }
                                    // };

                                    loop {
                                        let wait = spawn({
                                            // let wait_time = wait_time.clone();

                                            move || sleep(wait_time)
                                        });
                                        // let when = Instant::now() + wait_time;
                                        // let sample = auto.step();
                                        let s = auto.step();


                                        if let Err(e) = Python::attach({
                                            let func = func.clone();
                                            let args = args.clone();
                                            let kwargs = kwargs.clone();
                                        
                                            move |py| {
                                                let args = args.bind(py).clone().unbind();
                                                // let kwargs = kwargs.map(|kwargs| kwargs.bind(py).clone().unbind());
                                                let kwargs = kwargs.clone();

                                                let f = move |py, s| {
                                                    let kwargs = kwargs.deref().as_ref().map(|kwargs| kwargs.bind(py).to_owned());
                                                    let args = args.bind(py).to_list();

                                                    if let Err(e) = args.append(s as f32) {
                                                        println!("couldn't add sample to args list. {e}");
                                                    }
                                                    
                                                    let args = args.to_tuple();

                                                    // func.call(py, args, kwargs)
                                                    func.call(
                                                        py,
                                                        args,
                                                        kwargs.as_ref(), // .map(|kwargs| kwargs.bind(py).to_owned())
                                                                        // .as_ref(),
                                                    )
                                                    // Ok(())
                                                };

                                                f(py, s)
                                            }
                                        }) {
                                            println!(
                                                "running custom: {func_name}, resulted in error, {e}"
                                            );
                                            break;
                                        }                                    

                                        if let Err(e) = wait.join() {
                                            println!("wait thread for lfo sample gen timing failed with error, {e:?}");
                                        }
                                        // sleep_until(when);
                                    }
                                }
                            // })
                        })
                    })
                });

                Ok(())
            })
        }
    )
}

#[cfg(feature = "pyo3")]
#[pymodule]
#[pyo3(submodule, name = "lfo")]
/// A Python module implemented in Rust.
pub fn lfo_py(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // m.add_class::<MidiDaw>()?;
    m.add_function(wrap_pyfunction!(sin, m)?)?;

    Ok(())
}
