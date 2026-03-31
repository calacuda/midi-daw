use crate::automation::{AUTOMATIONS_PER_SECOND, AutomationTrait};
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
// use std::sync::Arc;

#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct SinLfo {
    last_sample: f64,
    freq: f64,
    one_shot: bool,
    bipolar: bool,
    hifi: bool,
    seen_zero: usize,
    step: f64,
    i: f64,
}

impl AutomationTrait for SinLfo {
    fn sub_type(&self) -> String {
        "sin".into()
    }

    fn update(&mut self) {
        self.last_sample = self.get_sample();
    }

    fn get_value(&self) -> f64 {
        self.last_sample
    }

    fn done(&self) -> bool {
        self.one_shot && (self.seen_zero % 3) == 0
    }
}

impl SinLfo {
    pub fn new(freq: f64, one_shot: bool, bipolar: bool, hifi: bool) -> Self {
        let step = freq / AUTOMATIONS_PER_SECOND;

        Self {
            last_sample: 0.0,
            seen_zero: 1,
            freq,
            one_shot,
            bipolar,
            hifi,
            i: 0.0,
            step,
        }
    }

    fn get_sample(&mut self) -> f64 {
        let sample = self.i.sin();

        self.i += self.step;

        if sample == 0.0 {
            self.seen_zero += 1;
        }

        sample
    }
}
