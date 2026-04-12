use std::f64::consts::PI;

use crate::v2::automation::AutomationTraitV2;
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
    sample_rate: f64,
    step: f64,
    i: f64,
}

impl AutomationTraitV2 for SinLfo {
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
        self.one_shot && self.seen_zero.is_multiple_of(3)
    }
}

impl SinLfo {
    pub fn new(freq: f64, one_shot: bool, bipolar: bool, hifi: bool, sample_rate: f64) -> Self {
        // let step = freq / AUTOMATIONS_PER_SECOND;
        // let step = sample_rate / freq / 60.;
        let step = (2. * PI * freq) / sample_rate;

        Self {
            last_sample: 0.0,
            seen_zero: 1,
            freq,
            one_shot,
            bipolar,
            hifi,
            sample_rate,
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
