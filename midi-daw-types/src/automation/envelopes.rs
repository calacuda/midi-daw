use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Clone, Debug)]
pub enum EnvConfig {
    /// Attack Decay Sustain Release.
    ADSR {
        atk: f64,
        dcy: f64,
        sus: f64,
        rel: f64,
    },
    /// Attack Release
    AR { atk: f64, rel: f64 },
}

