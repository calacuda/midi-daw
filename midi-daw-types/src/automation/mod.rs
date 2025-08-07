use enum_dispatch::enum_dispatch;
use lfo::{wavetable, Lfo};
use pyo3::{prelude::*, PyClass};
use serde::{Deserialize, Serialize};

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

impl From<AutomationConf> for AutomationTypes {
    fn from(value: AutomationConf) -> Self {
        match value {
            AutomationConf::Lfo(lfo::LfoConfig::WaveTable { file, freq }) => {
                // TODO: read wav file to Vec<f64>
                // TODO: build WaveTable
                // TODO: set WaveTable frequency to freq
                todo!("do this");
            }
            // AutomationConf::EnvelopeGen() => {}
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
    fn new(conf: AutomationConf) -> Self {
        let automation = AutomationTypes::from(conf);
        Self { automation }
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

    // fn sub_type(&self) -> String {
    // self.sub_type().into()
    // }
}
