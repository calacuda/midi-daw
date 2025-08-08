use enum_dispatch::enum_dispatch;
use hound::WavReader;
use lfo::{wavetable, Lfo};
use pyo3::{exceptions::PyValueError, prelude::*, PyClass};
use serde::{Deserialize, Serialize};

use crate::automation::lfo::wavetable::WaveTable;

// pub mod envelope;
pub mod lfo;

#[enum_dispatch]
pub trait AutomationTrait: PyClass {
    // fn automation_type(&self) -> impl Into<String>;
    fn sub_type(&self) -> impl Into<String>;
    /// used to update the state of the automation
    fn update(&mut self);
    /// used to get the last value of automation
    fn get_value(&self) -> f64;
    fn step(&mut self) -> f64 {
        self.update();
        self.get_value()
    }
}

#[pyclass]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
#[enum_dispatch(AutomationTrait)]
pub enum AutomationTypes {
    Lfo(lfo::Lfo),
    // EnvelopeGen(envelope::Envelope),
}

#[pyclass]
#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Clone, Debug)]
pub enum AutomationConf {
    Lfo(lfo::LfoConfig),
    // EnvelopeGen(envelope::EnvConfig),
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
                let samples = reader
                    .samples::<f32>()
                    .map(|sample| {
                        let sample = sample
                            .map_err(|e| format!("sample failed to decode with error: {e}"))?;
                        // Ok((sample as f64) / (i32::MAX as f64))
                        Ok(sample as f64)
                    })
                    .collect::<Result<Vec<f64>, String>>()?;

                // build WaveTable
                let mut wavetable =
                    WaveTable::new(samples.into(), reader.spec().sample_rate as f64);
                wavetable.set_frequency(freq);

                // set WaveTable frequency to freq
                Ok(AutomationTypes::Lfo(lfo::Lfo::WaveTable(wavetable)))
            } // AutomationConf::EnvelopeGen() => {}
        }
    }
}

#[pyclass]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct Automation {
    automation: AutomationTypes,
}

#[pymethods]
impl Automation {
    #[new]
    fn new(conf: AutomationConf) -> PyResult<Self> {
        let automation = match AutomationTypes::try_from(conf) {
            Ok(automation) => automation,
            Err(e) => {
                eprintln!("making automation failed with error: {e}");
                return Err(PyErr::new::<PyValueError, _>(e.to_string()));
            }
        };

        Ok(Self { automation })
    }

    fn step(&mut self) -> f64 {
        self.automation.step()
    }

    fn get_repr(&self) -> String {
        match self.automation.clone() {
            AutomationTypes::Lfo(lfo) => format!("lfo:{}", lfo.sub_type().into()),
            // AutomationTypes::EnvelopeGen(env) => format!("env:{}", env.sub_type().into()),
        }
    }

    // fn automation_type(&self) -> String {
    // self.automation_step().into()
    // }

    fn sub_type(&self) -> String {
        self.automation.sub_type().into()
    }
}
