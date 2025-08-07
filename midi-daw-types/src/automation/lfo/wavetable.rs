use crate::automation::AutomationTrait;
use pyo3::prelude::*;
use std::sync::Arc;

#[pyclass]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
pub struct WaveTable {
    sample_rate: f64,
    index: f64,
    index_increment: f64,
    wavetable: Arc<[f64]>,
    last_sample: f64,
}

impl AutomationTrait for WaveTable {
    fn sub_type(&self) -> impl Into<String> {
        "wavetable"
    }

    fn update(&mut self) {
        self.last_sample = self.get_sample();
    }

    fn get_value(&self) -> f64 {
        self.last_sample
    }
}

impl WaveTable {
    pub fn new(wavetable: Vec<f64>, sample_rate: f64) -> Self {
        let wavetable = wavetable.into();

        Self {
            sample_rate,
            index: 0.0,
            index_increment: 0.0,
            wavetable,
            last_sample: 0.0,
        }
    }

    pub fn set_frequency(&mut self, frequency: f64) {
        self.index_increment = frequency * self.wavetable.len() as f64 / self.sample_rate;
        self.index = 0.0;
    }

    pub fn get_sample(&mut self) -> f64 {
        let sample = self.lerp();

        self.index += self.index_increment;
        self.index %= self.wavetable.len() as f64;

        sample * 0.9
    }

    /// steps forward by n samples with out computing a sample
    pub fn step_forward(&mut self, n: usize) {
        // let mut sample = 0.0;

        // sample += self.lerp();

        self.index += self.index_increment * n as f64;
        self.index %= self.wavetable.len() as f64;

        // sample * 0.9
    }

    fn lerp(&self) -> f64 {
        let truncated_index = self.index as usize;
        let next_index = (truncated_index + 1) % self.wavetable.len();
        let next_index_weight = self.index - truncated_index as f64;
        let truncated_index_weight = 1.0 - next_index_weight;

        truncated_index_weight * self.wavetable[truncated_index]
            + next_index_weight * self.wavetable[next_index]
    }
}
