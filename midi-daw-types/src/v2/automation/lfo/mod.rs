use bincode::{Decode, Encode};
// use super::AutomationTrait;
use enum_dispatch::enum_dispatch;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use crate::v2::automation::AutomationTraitV2;

pub mod sin_wave;
pub mod wavetable;

// #[pyclass]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, PartialOrd, Clone, Debug)]
pub enum LfoConfig {
    /// wave-table lfo
    WaveTable { file: String, freq: f64 },
    /// sin wave
    Sin {
        freq: f64,
        one_shot: bool,
        bipolar: bool,
        hifi: bool,
        sample_rate: f64,
    },
    // /// triangle wave
    // Triangle {
    //     freq: f64,
    //     one_shot: bool,
    //     bipolar: bool,
    //     hifi: bool,
    // },
    // /// saw wave going up
    // SawUp {
    //     freq: f64,
    //     one_shot: bool,
    //     bipolar: bool,
    //     hifi: bool,
    // },
    // /// saw wave going down
    // SawDown {
    //     freq: f64,
    //     one_shot: bool,
    //     bipolar: bool,
    //     hifi: bool,
    // },
    // /// anti-log Triangle Wave
    // AntiLog {
    //     freq: f64,
    //     one_shot: bool,
    //     bipolar: bool,
    //     hifi: bool,
    // },
    // /// anti-log saw wave going up
    // AntiLogUp {
    //     freq: f64,
    //     one_shot: bool,
    //     bipolar: bool,
    //     hifi: bool,
    // },
    // /// anti-log saw wave going down
    // AntiLogDown {
    //     freq: f64,
    //     one_shot: bool,
    //     bipolar: bool,
    //     hifi: bool,
    // },
}

// #[pyclass]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
#[enum_dispatch(AutomationTraitV2)]
pub enum Lfo {
    /// wave-table lfo
    WaveTable(wavetable::WaveTable),
    /// sin wave
    Sin(sin_wave::SinLfo),
    // /// triangle wave
    // Triangle {
    //     freq: f64,
    //     one_shot: bool,
    //     bipolar: bool,
    //     hifi: bool,
    // },
    // /// saw wave going up
    // SawUp {
    //     freq: f64,
    //     one_shot: bool,
    //     bipolar: bool,
    //     hifi: bool,
    // },
    // /// saw wave going down
    // SawDown {
    //     freq: f64,
    //     one_shot: bool,
    //     bipolar: bool,
    //     hifi: bool,
    // },
    // /// anti-log Triangle Wave
    // AntiLog {
    //     freq: f64,
    //     one_shot: bool,
    //     bipolar: bool,
    //     hifi: bool,
    // },
    // /// anti-log saw wave going up
    // AntiLogUp {
    //     freq: f64,
    //     one_shot: bool,
    //     bipolar: bool,
    //     hifi: bool,
    // },
    // /// anti-log saw wave going down
    // AntiLogDown {
    //     freq: f64,
    //     one_shot: bool,
    //     bipolar: bool,
    //     hifi: bool,
    // },
}
