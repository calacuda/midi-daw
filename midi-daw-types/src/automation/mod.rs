use bincode::{Decode, Encode};
use enum_dispatch::enum_dispatch;
use hound::{SampleFormat, WavReader};
use lfo::{Lfo, sin_wave, wavetable};
#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, types::PyDict};
use serde::{Deserialize, Serialize};

use crate::automation::lfo::{sin_wave::SinLfo, wavetable::WaveTable};

// pub mod envelope;
pub mod lfo;

pub static AUTOMATIONS_PER_SECOND: f64 = 48_000.0;

#[enum_dispatch]
pub trait AutomationTrait /*: PyClass */ {
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
#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
#[enum_dispatch(AutomationTrait)]
pub enum AutomationTypes {
    Lfo(lfo::Lfo),
    // EnvelopeGen(envelope::Envelope),
}

// #[pyclass]
#[cfg_attr(feature = "pyo3", pyclass)]
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
            }) => Ok(AutomationTypes::Lfo(lfo::Lfo::Sin(SinLfo::new(
                freq, one_shot, bipolar, hifi,
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
