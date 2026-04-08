#![feature(thread_sleep_until)]
use bincode::{
    Decode, Encode,
    error::{DecodeError, EncodeError},
};
#[cfg(feature = "pyo3")]
use log::*;
use midi_msg::Channel;
#[cfg(feature = "pyo3")]
use pyo3::{prelude::*, types::PyDict};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[cfg(not(feature = "pyo3"))]
use tracing::*;

use crate::v2::DEFAULT_BPM;

pub const UDS_SERVER_PATH: &str = "/tmp/midi-daw.sock";

pub type MidiDeviceName = String;
pub type Tempo = Arc<std::sync::RwLock<f64>>;

pub mod v1;
pub mod v2;

#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
#[derive(Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone)]
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

// #[cfg(feature = "pyo3")]
// #[pymethods]
#[cfg_attr(feature = "pyo3", pymethods)]
impl MidiTarget {
    #[cfg(feature = "pyo3")]
    #[new]
    fn new_py() -> Self {
        Self::new()
    }

    #[cfg(feature = "pyo3")]
    // This method enables Python's copy.deepcopy()
    pub fn __deepcopy__(&self, _memo: &Bound<'_, PyDict>) -> Self {
        self.clone() // Use Rust's Clone implementation
    }
}

impl MidiTarget {
    pub fn new() -> Self {
        Self::default()
    }
}

// #[pyclass]
#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(
    Serialize,
    Deserialize,
    Encode,
    Decode,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Clone,
    Copy,
    Debug,
)]
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

impl MidiChannel {
    pub fn new() -> Self {
        Self::default()
    }

    // #[staticmethod]
    pub fn do_from_hex(hex: String) -> Self {
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

    pub fn do_from_int(n: isize) -> Self {
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

// #[cfg_attr(feature = "pyo3", pymethods)]
#[cfg(feature = "pyo3")]
#[pymethods]
impl MidiChannel {
    #[cfg(feature = "pyo3")]
    #[new]
    fn new_py() -> Self {
        Self::new()
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
    // #[cfg_attr(feature = "pyo3", staticmethod)]
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
    // #[cfg_attr(feature = "pyo3", staticmethod)]
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

    // This method enables Python's copy.deepcopy()
    pub fn __deepcopy__(&self, _memo: &Bound<'_, PyDict>) -> Self {
        self.clone() // Use Rust's Clone implementation
    }
}

#[cfg_attr(feature = "pyo3", pyclass(name = "NoteLen", from_py_object))]
#[derive(
    Serialize, Deserialize, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash,
)]
pub enum NoteDuration {
    // how_many: u8
    Wn(u8),
    Hn(u8),
    Qn(u8),
    En(u8),
    Sn(u8),
    Tn(u8),
    S4n(u8),
}

impl Default for NoteDuration {
    fn default() -> Self {
        Self::Sn(1)
    }
}

impl NoteDuration {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn json(&self) -> String {
        let Ok(res) = serde_json::to_string(self) else {
            return String::new();
        };

        res
    }
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl NoteDuration {
    #[cfg(feature = "pyo3")]
    #[new]
    fn new_py() -> Self {
        Self::new()
    }

    // pub fn __str__(&self) -> String {
    //     match *self {
    //     }
    // }

    // #[staticmethod]
    // pub fn from_str(str_repr: String) -> Self {
    //     let str_repr = str_repr.to_lowercase();
    //
    //
    // }

    #[pyo3(name = "json")]
    fn json_py(&self) -> String {
        self.json()
    }
}

#[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum MidiMsg {
    PlayNote {
        // target: MidiTarget,
        // note: MidiNote,
        note: u8,
        velocity: u8,
        duration: NoteDuration,
    },
    StopNote {
        // target: MidiTarget,
        note: u8,
    },
    PitchBend {
        bend: u16,
    },
    CC {
        control: u8,
        value: u8,
    },
    // TODO: add a Panic message.
    // TODO: consider adding the bellow messages
    //
    // ModWheel { amt: u16 },
    // Volume { amt: u16 },
}

// #[pymethods]
#[cfg_attr(feature = "pyo3", pymethods)]
impl MidiMsg {
    fn __repr__(&self) -> String {
        format!("{self:?}")
    }
}

impl From<MidiChannel> for Channel {
    fn from(value: MidiChannel) -> Self {
        match value {
            MidiChannel::Ch1 => Channel::Ch1,
            MidiChannel::Ch2 => Channel::Ch2,
            MidiChannel::Ch3 => Channel::Ch3,
            MidiChannel::Ch4 => Channel::Ch4,
            MidiChannel::Ch5 => Channel::Ch5,
            MidiChannel::Ch6 => Channel::Ch6,
            MidiChannel::Ch7 => Channel::Ch7,
            MidiChannel::Ch8 => Channel::Ch8,
            MidiChannel::Ch9 => Channel::Ch9,
            MidiChannel::Ch10 => Channel::Ch10,
            MidiChannel::Ch11 => Channel::Ch11,
            MidiChannel::Ch12 => Channel::Ch12,
            MidiChannel::Ch13 => Channel::Ch13,
            MidiChannel::Ch14 => Channel::Ch14,
            MidiChannel::Ch15 => Channel::Ch15,
            MidiChannel::Ch16 => Channel::Ch16,
        }
    }
}

pub fn get_bincode_conf() -> bincode::config::Configuration {
    bincode::config::standard()
}

pub fn tempo_from_bpm(bpm: u32) -> u32 {
    (1_000_000 * 60) / DEFAULT_BPM
}

#[cfg(feature = "pyo3")]
/// A Python module implemented in Rust.
#[pymodule]
#[pyo3(name = "midi_daw")]
fn midi_daw_types(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // pyo3_log::init();

    m.add_class::<MidiChannel>()?;
    m.add_class::<MidiTarget>()?;
    m.add_class::<MidiMsg>()?;
    m.add_class::<NoteDuration>()?;

    // v1
    {
        let module = PyModule::new(py, "v1")?;
        v1::v1(&module)?;
        m.add_submodule(&module)?;
        py.import("sys")?
            .getattr("modules")?
            .set_item("midi_daw.v1", &module)?;
    }

    // v2
    {
        let module = PyModule::new(py, "v2")?;
        v2::v2(py, &module)?;
        m.add_submodule(&module)?;
        py.import("sys")?
            .getattr("modules")?
            .set_item("midi_daw.v2", &module)?;
    }

    Ok(())
}
