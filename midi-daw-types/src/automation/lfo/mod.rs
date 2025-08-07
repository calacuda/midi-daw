use super::AutomationTrait;
use enum_dispatch::enum_dispatch;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

pub mod wavetable;

#[pyclass]
#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Clone, Debug)]
pub enum LfoConfig {
    /// wave-table lfo
    WaveTable { file: String, freq: f64 },
    // /// sin wave
    // Sin {
    //     freq: f64,
    //     one_shot: bool,
    //     bipolar: bool,
    //     hifi: bool,
    // },
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

#[pyclass]
#[derive(PartialEq, PartialOrd, Clone, Debug)]
#[enum_dispatch(AutomationTrait)]
pub enum Lfo {
    /// wave-table lfo
    WaveTable(wavetable::WaveTable),
    // /// sin wave
    // Sin(SinLfo),
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
