use std::time::Duration;

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

#[pymethods]
impl MidiTarget {
    #[new]
    fn new() -> Self {
        Self::default()
    }
}

#[pyclass]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy, Debug)]
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

#[pymethods]
impl MidiChannel {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn __str__(&self) -> String {
        match *self {
            Self::Ch1 => "1".into(),
            Self::Ch2 => "2".into(),
            Self::Ch3 => "3".into(),
            Self::Ch4 => "4".into(),
            Self::Ch5 => "5".into(),
            Self::Ch6 => "6".into(),
            Self::Ch7 => "7".into(),
            Self::Ch8 => "8".into(),
            Self::Ch9 => "9".into(),
            Self::Ch10 => "10".into(),
            Self::Ch11 => "11".into(),
            Self::Ch12 => "12".into(),
            Self::Ch13 => "13".into(),
            Self::Ch14 => "14".into(),
            Self::Ch15 => "15".into(),
            Self::Ch16 => "16".into(),
        }
    }

    #[staticmethod]
    pub fn from_hex(hex: String) -> Self {
        let hex = hex.to_lowercase();
        let hex = if hex.starts_with("0x") {
            hex.replace("0x", "")
        } else {
            hex
        };

        match hex.as_str() {
            "0" => Self::Ch1,
            "1" => Self::Ch2,
            "2" => Self::Ch3,
            "3" => Self::Ch4,
            "4" => Self::Ch5,
            "5" => Self::Ch6,
            "6" => Self::Ch7,
            "7" => Self::Ch8,
            "8" => Self::Ch9,
            "9" => Self::Ch10,
            "a" => Self::Ch11,
            "b" => Self::Ch12,
            "c" => Self::Ch13,
            "d" => Self::Ch14,
            "e" => Self::Ch15,
            "f" => Self::Ch16,
            // _ => Err(format!("{hex} is ether not valid hex, or not between 0x0 & 0xF").into()),
            _ => Self::Ch1,
        }
    }

    #[staticmethod]
    pub fn from_int(n: isize) -> Self {
        if !(1..=16).contains(&n) {
            return Self::Ch1;
        }

        let channels = [
            Self::Ch1,
            Self::Ch2,
            Self::Ch3,
            Self::Ch4,
            Self::Ch5,
            Self::Ch6,
            Self::Ch7,
            Self::Ch8,
            Self::Ch9,
            Self::Ch10,
            Self::Ch11,
            Self::Ch12,
            Self::Ch13,
            Self::Ch14,
            Self::Ch15,
            Self::Ch16,
        ];

        channels[(n - 1) as usize]
    }
}

#[pyclass]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum MidiNote {
    C0,
}

#[pyclass]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum MidiMsg {
    PlayNote {
        target: MidiTarget,
        note: MidiNote,
        velocity: u8,
        duration: Duration,
    },
    StopNote {
        target: MidiTarget,
        note: u8,
    },
}

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
    m.add_class::<MidiMsg>()?;
    m.add_class::<MidiNote>()?;

    // m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}
