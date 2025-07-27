use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

pub type MidiDeviceName = String;

#[pyclass]
#[pyo3(get_all, set_all)]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct MidiTarget {
    pub name: MidiDeviceName,
    pub ch: MidiChannel,
}

impl Default for MidiTarget {
    fn default() -> Self {
        Self {
            name: "MIDI THRU".into(),
            ch: MidiChannel::default(),
        }
    }
}

#[pyclass]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy)]
pub enum MidiChannel {
    #[default]
    Ch1,
    Ch2,
    Ch3,
    Ch4,
    Ch5,
    Ch6,
    Ch7,
    Ch8,
    Ch9,
    Ch10,
    Ch11,
    Ch12,
    Ch13,
    Ch14,
    Ch15,
    Ch16,
}

// #[pymethods]
// impl MidiChannel {
//
// }

// /// Formats the sum of two numbers as string.
// #[pyfunction]
// fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
//     Ok((a + b).to_string())
// }

/// A Python module implemented in Rust.
#[pymodule]
fn midi_daw_types(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<MidiChannel>()?;
    m.add_class::<MidiTarget>()?;

    // m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}
