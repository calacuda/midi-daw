use enum_dispatch::enum_dispatch;
use pyo3::{prelude::*, PyClass};
use serde::{Deserialize, Serialize};

pub mod envelopes;
pub mod lfo;

#[enum_dispatch]
pub trait AutomationTrait: PyClass {
    // fn automation_type(&self) -> impl Into<String>;
    fn sub_type(&self) -> impl Into<String>;
    fn update(&mut self);
    fn get_value(&self) -> f64;
    fn automation_step(&mut self) -> f64 {
        self.update();
        self.get_value()
    }
}

#[pyclass]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
#[enum_dispatch(AutomationTrait)]
pub enum AutomationTypes {
    Lfo(lfo::Lfo),
    EnvelopeGen(envelopes::Envelope),
}

#[pyclass]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum AutomationConf {
    Lfo(lfo::LFOConf),
    EnvelopeGen(envelopes::EnvelopeConf),
}

impl From<AutomationConf> for AutomationTypes {
    fn from(value: AutomationConf) -> Self {
        match value {
            // AutomationConf::Lfo() => {}
            // AutomationConf::EnvelopeGen() => {}
        }
    }
}

#[pyclass]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
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
        self.automation.automation_step()
    }

    fn get_repr(&self) -> String {
        match self.automation.clone() {
            AutomationTypes::Lfo(lfo) => format!("lfo:{}", lfo.sub_type()),
            AutomationTypes::EnvelopeGen(env) => format!("env:{}", env.sub_type()),
        }
    }

    // fn automation_type(&self) -> String {
    // self.automation_step().into()
    // }

    // fn sub_type(&self) -> String {
    // self.sub_type().into()
    // }
}
