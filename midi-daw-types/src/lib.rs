#[cfg(feature = "pyo3")]
use crate::automation::{Automation, AutomationConf, AutomationTypes, lfo::LfoConfig};
use midi_msg::Channel;
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub const UDS_SERVER_PATH: &str = "/tmp/midi-daw.sock";

pub type MidiDeviceName = String;
pub type Tempo = Arc<std::sync::RwLock<f64>>;
pub type BPQ = Arc<std::sync::RwLock<f64>>;

pub mod automation;

#[cfg_attr(feature = "pyo3", pyclass)]
#[cfg_attr(feature = "pyo3", pyo3(get_all, set_all))]
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

// #[cfg(feature = "pyo3")]
// #[pymethods]
#[cfg_attr(feature = "pyo3", pymethods)]
impl MidiTarget {
    #[cfg(feature = "pyo3")]
    #[new]
    fn new_py() -> Self {
        Self::new()
    }
}

impl MidiTarget {
    pub fn new() -> Self {
        Self::default()
    }
}

// #[pyclass]
#[cfg_attr(feature = "pyo3", pyclass)]
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
}

// #[pyclass]
// #[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
// pub enum MidiNote {
//     C(u8),
//     C#(u8),
// }

// #[pyclass]
// #[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy, Debug)]
// pub enum MidiChannel {
//
// }

// #[pyclass]
// #[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Copy, Debug)]
// pub enum AutomationConf {
//     LFO {
//         freq: f64,
//
//     }
// }

#[cfg_attr(feature = "pyo3", pyclass(name = "NoteLen"))]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
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

#[cfg_attr(feature = "pyo3", pyclass)]
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

// /// Formats the sum of two numbers as string.
// #[pyfunction]
// fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
//     Ok((a + b).to_string())
// }

/// gets midi note as a u8 from a string name.
#[cfg_attr(feature = "pyo3", pyfunction)]
pub fn note_from_str(name: String) -> Option<u8> {
    let mut name = name.to_lowercase().replace(" ", "");
    // let mut octave = 24;

    let scale_offset: u8 = if name.starts_with("b#") || name.starts_with("cb") {
        print!("[Error] Sorry but \"{name}\" is not a real note");
        return None;
    } else if name.starts_with("c#") || name.starts_with("db") {
        name = name.replace("c#", "").replace("db", "");
        1
    } else if name.starts_with("d#") || name.starts_with("eb") {
        name = name.replace("d#", "").replace("eb", "");
        3
    } else if name.starts_with("e#") || name.starts_with("fb") {
        print!("[Error] Sorry but \"{name}\" is not a real note");
        return None;
    } else if name.starts_with("f#") || name.starts_with("gb") {
        name = name.replace("f#", "").replace("gb", "");
        6
    } else if name.starts_with("g#") || name.starts_with("ab") {
        name = name.replace("g#", "").replace("ab", "");
        8
    } else if name.starts_with("a#") || name.starts_with("bb") {
        name = name.replace("a#", "").replace("bb", "");
        10
    } else if name.starts_with("c") {
        name = name.replace("c", "");
        0
    } else if name.starts_with("d") {
        name = name.replace("d", "");
        2
    } else if name.starts_with("e") {
        name = name.replace("e", "");
        4
    } else if name.starts_with("f") {
        name = name.replace("f", "");
        5
    } else if name.starts_with("g") {
        name = name.replace("g", "");
        7
    } else if name.starts_with("a") {
        name = name.replace("a", "");
        9
    } else if name.starts_with("b") {
        name = name.replace("b", "");
        11
    } else {
        print!("[Error] Sorry but \"{name}\" is not a real note");
        return None;
    };

    // println!("name -> {name}");

    let octave: u8 = if let Ok(octave) = name.parse()
        && !name.is_empty()
    {
        octave
    } else {
        1
    };

    Some(12 * octave + scale_offset + 12)
}

// #[pyclass]
#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct MidiReqBody {
    pub midi_dev: String,
    pub channel: MidiChannel,
    pub msg: MidiMsg,
}

impl MidiReqBody {
    pub fn new(midi_dev: String, channel: MidiChannel, msg: MidiMsg) -> Self {
        Self {
            midi_dev,
            channel,
            msg,
        }
    }

    pub fn json(&self) -> String {
        let Ok(res) = serde_json::to_string(self) else {
            return String::new();
        };

        res
    }
}

pub fn display_midi_note(midi_note: u8) -> String {
    let note_name_i = midi_note % 12;
    let octave = midi_note / 12;

    let note_names = [
        "C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-",
    ];
    let note_name = note_names[note_name_i as usize];

    format!("{note_name}{octave:X}")
}

#[cfg(feature = "pyo3")]
#[pymethods]
// #[cfg_attr(feature = "pyo3", pymethods)]
impl MidiReqBody {
    #[new]
    fn new_py(midi_dev: String, channel: MidiChannel, msg: MidiMsg) -> Self {
        Self::new(midi_dev, channel, msg)
    }

    // #[cfg_attr(feature = "pyo3", new)]
    #[pyo3(name = "json")]
    fn json_py(&self) -> String {
        self.json()
    }
}

// // #[pyclass]
// #[cfg_attr(feature = "pyo3", pyclass)]
// #[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
// pub struct RestReqBody {
//     pub tempo: String,
//     // pub channel: MidiChannel,
//     // pub msg: MidiMsg
// }

#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct AddNoteBody {
    pub sequence: String,
    pub step: usize,
    pub note: u8,
    pub velocity: u8,
    pub note_len: Option<NoteDuration>,
}

impl AddNoteBody {
    pub fn new(
        sequence: String,
        step: usize,
        note: u8,
        velocity: u8,
        note_len: Option<NoteDuration>,
    ) -> Self {
        Self {
            sequence,
            step,
            note,
            velocity,
            note_len,
        }
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
impl AddNoteBody {
    #[new]
    fn new_py(
        sequence: String,
        step: usize,
        note: u8,
        velocity: u8,
        note_len: Option<NoteDuration>,
    ) -> Self {
        Self::new(sequence, step, note, velocity, note_len)
    }

    #[pyo3(name = "json")]
    fn json_py(&self) -> String {
        self.json()
    }
}

#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct RmNoteBody {
    pub sequence: String,
    pub step: usize,
    pub note: u8,
}

impl RmNoteBody {
    pub fn new(sequence: String, step: usize, note: u8) -> Self {
        Self {
            sequence,
            step,
            note,
        }
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
impl RmNoteBody {
    #[new]
    fn new_py(sequence: String, step: usize, note: u8) -> Self {
        Self::new(sequence, step, note)
    }

    #[pyo3(name = "json")]
    fn json_py(&self) -> String {
        self.json()
    }
}

#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct SetDevBody {
    pub sequence: String,
    pub midi_dev: String,
}

impl SetDevBody {
    pub fn new(sequence: String, midi_dev: String) -> Self {
        Self { sequence, midi_dev }
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
impl SetDevBody {
    #[new]
    fn new_py(sequence: String, midi_dev: String) -> Self {
        Self::new(sequence, midi_dev)
    }

    #[pyo3(name = "json")]
    fn json_py(&self) -> String {
        self.json()
    }
}

#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct GetSequenceQuery {
    pub sequence: String,
    // pub midi_dev: String,
}

impl GetSequenceQuery {
    pub fn new(sequence: String) -> Self {
        Self { sequence }
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
impl GetSequenceQuery {
    #[new]
    fn new_py(sequence: String) -> Self {
        Self::new(sequence)
    }

    #[pyo3(name = "json")]
    fn json_py(&self) -> String {
        self.json()
    }
}

#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct RenameSequenceBody {
    pub old_name: String,
    pub new_name: String,
}

impl RenameSequenceBody {
    pub fn new(old_name: String, new_name: String) -> Self {
        Self { old_name, new_name }
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
impl RenameSequenceBody {
    #[new]
    fn new_py(old_name: String, new_name: String) -> Self {
        Self::new(old_name, new_name)
    }

    #[pyo3(name = "json")]
    fn json_py(&self) -> String {
        self.json()
    }
}

#[cfg_attr(feature = "pyo3", pyclass)]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct SetChannelBody {
    pub sequence: String,
    pub channel: MidiChannel,
}

impl SetChannelBody {
    pub fn new(sequence: String, channel: MidiChannel) -> Self {
        Self { sequence, channel }
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
impl SetChannelBody {
    #[new]
    fn new_py(sequence: String, channel: MidiChannel) -> Self {
        Self::new(sequence, channel)
    }

    #[pyo3(name = "json")]
    fn json_py(&self) -> String {
        self.json()
    }
}

// impl Into<Channel> for MidiChannel {
//     fn into(self) -> Channel {
//         match self {
//             Self::Ch1 => Channel::Ch1,
//             Self::Ch2 => Channel::Ch2,
//             Self::Ch3 => Channel::Ch3,
//             Self::Ch4 => Channel::Ch4,
//             Self::Ch5 => Channel::Ch5,
//             Self::Ch6 => Channel::Ch6,
//             Self::Ch7 => Channel::Ch7,
//             Self::Ch8 => Channel::Ch8,
//             Self::Ch9 => Channel::Ch9,
//             Self::Ch10 => Channel::Ch10,
//             Self::Ch11 => Channel::Ch11,
//             Self::Ch12 => Channel::Ch12,
//             Self::Ch13 => Channel::Ch13,
//             Self::Ch14 => Channel::Ch14,
//             Self::Ch15 => Channel::Ch15,
//             Self::Ch16 => Channel::Ch16,
//         }
//     }
// }

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

#[cfg(feature = "pyo3")]
/// A Python module implemented in Rust.
#[pymodule]
fn midi_daw_types(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<MidiChannel>()?;
    m.add_class::<MidiTarget>()?;
    m.add_class::<MidiMsg>()?;
    m.add_class::<NoteDuration>()?;
    m.add_class::<MidiReqBody>()?;

    m.add_class::<Automation>()?;
    m.add_class::<AutomationTypes>()?;
    m.add_class::<AutomationConf>()?;
    m.add_class::<LfoConfig>()?;
    // m.add_class::<LfoConfig>()?;

    // m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(note_from_str, m)?)?;
    m.add("UDS_SERVER_PATH", UDS_SERVER_PATH)?;

    Ok(())
}
