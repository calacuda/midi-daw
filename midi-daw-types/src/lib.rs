use midi_msg::Channel;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

pub const UDS_SERVER_PATH: &str = "/tmp/midi-daw.sock";

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

#[pyclass(name = "NoteLen")]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
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

#[pymethods]
impl NoteDuration {
    #[new]
    pub fn new() -> Self {
        Self::default()
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

    fn json(&self) -> String {
        let Ok(res) = serde_json::to_string(self) else {
            return String::new();
        };

        res
    }
}

#[pyclass]
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
    // TODO: consider adding the bellow messages
    //
    // ModWheel { amt: u16 },
    // Volume { amt: u16 },
}

#[pymethods]
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
#[pyfunction]
fn note_from_str(name: String) -> Option<u8> {
    let mut name = name.to_lowercase().replace(" ", "");
    // let mut octave = 24;

    let scale_offset: u8 = if name.starts_with("b#") || name.starts_with("cb") {
        print!("[Error] Sorry but \"{name}\" is not a real note");
        return None
    } else if name.starts_with("c#") || name.starts_with("db") {
        name = name.replace("c#", "").replace("db", "");
        1
    } else if name.starts_with("d#") || name.starts_with("eb") {
        name = name.replace("d#", "").replace("eb", "");
        3
    } else if name.starts_with("e#") || name.starts_with("fb") {
        print!("[Error] Sorry but \"{name}\" is not a real note");
        return None
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

    let octave: u8 = if let Ok(octave) = name.parse() && !name.is_empty() {
        octave
    } else { 
        1
    };

    Some(12 * octave + scale_offset + 12)
}

#[pyclass]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct MidiReqBody {
    pub midi_dev: String,
    pub channel: MidiChannel,
    pub msg: MidiMsg
}

#[pymethods]
impl MidiReqBody {
    #[new]
    fn new(midi_dev: String, channel: MidiChannel, msg: MidiMsg) -> Self {
        Self {
            midi_dev,
            channel,
            msg
        }

    }

    fn json(&self) -> String {
        let Ok(res) = serde_json::to_string(self) else {
            return String::new();
        };

        res
    }
}

#[pyclass]
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct RestReqBody {
    pub tempo: String,
    // pub channel: MidiChannel,
    // pub msg: MidiMsg
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

/// A Python module implemented in Rust.
#[pymodule]
fn midi_daw_types(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<MidiChannel>()?;
    m.add_class::<MidiTarget>()?;
    m.add_class::<MidiMsg>()?;
    m.add_class::<NoteDuration>()?;
    m.add_class::<MidiReqBody>()?;

    // m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(note_from_str, m)?)?;
    m.add("UDS_SERVER_PATH", UDS_SERVER_PATH)?;

    Ok(())
}
