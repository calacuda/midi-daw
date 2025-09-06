use crate::{button_tracker::ButtonPressTimers, helpers::less_then::UsizeLessThan};
use bevy::prelude::*;
use crossbeam::channel::Sender;
use midi_daw::midi::MidiDev;
use midi_daw_types::MidiDeviceName;
use midi_msg::{Channel, MidiMsg};
use std::{fmt::Display, time::Duration};
use strum::{EnumDiscriminants, EnumString};

pub mod button_tracker;
pub mod display;
pub mod helpers;
pub mod midi_plugin;
pub mod sphere;

pub type MidiNote = u8;

pub const N_STEPS: usize = 16;
pub const COL_W: usize = 18;

#[derive(Clone, DerefMut, Deref, Resource)]
pub struct NewMidiDev(pub Sender<MidiDev>);

impl NewMidiDev {
    pub fn create_new(&mut self, dev_name: MidiDeviceName) {
        if let Err(e) = self.0.send(MidiDev::CreateVirtual(dev_name.clone())) {
            error!("failed to send message to create virtual dev {dev_name}. {e}");
        }
    }
}

#[derive(Clone, DerefMut, Deref, Resource)]
pub struct MidiOutput(pub Sender<(String, MidiMsg)>);

impl MidiOutput {
    pub fn send(&mut self, dev: MidiDeviceName, msg: MidiMsg) {
        if let Err(e) = self.0.send((dev.clone(), msg)) {
            error!("failed to send message to device {dev}. {e}");
        }
    }
}

#[derive(Clone, Copy, Default, Debug, States, PartialEq, Eq, Hash)]
pub enum MainState {
    #[default]
    StartUp,
    Edit,
    ShutDown,
}

// #[derive(Clone, Default, Debug, Hash, PartialEq, Eq, States)]
#[derive(Clone, Default, Debug, PartialEq, Eq, Resource, Event, EnumDiscriminants)]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(name(ScreenState))]
#[strum_discriminants(derive(States, Default, Hash, PartialOrd, Ord))]
pub enum Screen {
    #[default]
    #[strum_discriminants(default)]
    MainScreen,
    // ChangeMidiDev {
    //     track_id: usize,
    //     old_dev: String,
    // },
    EditCmd {
        track_id: usize,
        step: UsizeLessThan<{ N_STEPS }>,
        cmd_n: UsizeLessThan<2>,
    },
    AddTrack,
    EditTrackMidiDev {
        track: usize,
        dev: Option<MidiDeviceName>,
        chan: Option<Channel>,
        // chan: UsizeLessThan<16>,
    },
    ChangeTempo {
        old_tempo: u16,
        new_tempo: u16,
    },
}

#[derive(Resource, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Deref, DerefMut)]
pub struct Playing(pub bool);

#[derive(Resource, Clone, Eq, PartialEq, PartialOrd, Ord, Deref, DerefMut)]
pub struct TrackFriendlyName(pub String);

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Deref, DerefMut)]
pub struct ColumnId(pub usize);

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Deref, DerefMut)]
pub struct RowId(pub usize);

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct PlayingMarker;

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct DevDisplay;

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct CellMarker {
    displayed_track_idx: usize,
    column: usize,
    row: usize,
}

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct LineNumMarker {
    track: u8,
    row: u8,
}

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Deref, DerefMut)]
pub struct TitleMarker(pub u8);

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct CursorText;

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Deref, DerefMut)]
pub struct CursorID(usize);

#[derive(Resource, Default, Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct CursorLocation(pub usize, pub usize);

#[derive(Resource, Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct DisplayStart(pub usize);

#[derive(Clone, Copy, Default, Debug, States, PartialEq, Eq, Hash, Resource, Deref, DerefMut)]
pub struct CmdPallet(pub bool);

#[derive(Clone, Copy, Default, Debug, States, PartialEq, Eq, Hash, Resource, Deref, DerefMut)]
pub struct EdittingCell(pub bool);

#[derive(Clone, Copy, Default, Debug, States, PartialEq, Eq, Hash, Resource, Deref, DerefMut)]
pub struct Tempo(pub u16);

#[derive(Clone, Debug, Component, PartialEq)]
pub enum Track {
    Midi {
        steps: Vec<Step<MidiCmd>>,
        dev: MidiDeviceName,
        chan: Channel,
    },
    SF2 {
        steps: Vec<Step<Sf2Cmd>>,
        dev: MidiDeviceName,
        chan: Channel,
    },
}

impl Default for Track {
    fn default() -> Self {
        Self::Midi {
            steps: (0..N_STEPS).map(|_| Step::default()).collect(),
            dev: MidiDeviceName::default(),
            chan: Channel::Ch1,
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, PartialOrd)]
pub struct Step<Cmd>
where
    Cmd:
        Clone + Default + PartialEq + PartialOrd + core::fmt::Display + ToString + core::fmt::Debug,
{
    pub note: Option<MidiNote>,
    pub cmds: (TrackerCmd<Cmd>, TrackerCmd<Cmd>),
}

#[derive(Clone, Copy, Default, Debug, PartialEq, PartialOrd, Eq, Hash)]
pub enum Intervals {
    #[default]
    Root,
    MajThird,
    MinThird,
    FlatFifth,
    Fifth,
    SharpFifth,
    FlatSeventh,
    Seventh,
    SharpSeventh,
}

#[derive(
    Clone, Default, Debug, PartialEq, Eq, PartialOrd, Hash, EnumString, strum_macros::Display,
)]
pub enum TrackerCmd<Cmd>
where
    Cmd: Clone + Default + PartialEq + PartialOrd + ToString + Display,
{
    #[default]
    #[strum(to_string = "----")]
    None,
    #[strum(to_string = "Chrd")]
    Chord { chord: Vec<Intervals> },
    #[strum(to_string = "Roll")]
    Roll {
        /// how many extra times to "roll" what ever is being played. a value of 1 would produce
        /// two 64th notes.
        times: usize,
    },
    // NOTE: maybe remove Swing
    #[strum(to_string = "Swng")]
    Swing {
        /// the amount of swing to put on the note
        amt: UsizeLessThan<128>,
    },
    #[strum(to_string = "Hold")]
    HoldFor {
        notes: UsizeLessThan<{ N_STEPS + 1 }>,
    },
    /// stop all notes on device
    #[strum(to_string = "Stop")]
    Panic,
    #[strum(transparent)]
    Custom(Cmd),
}

// TODO: impl Display for TrackerCmd

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct MidiCmd {
    cc_param: u8,
    arg_1: u8,
    arg_2: u8,
}

impl Display for MidiCmd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CC--")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, EnumString, strum_macros::Display)]
pub enum Sf2Cmd {
    #[strum(to_string = "Atk-")]
    Atk(usize),
    #[strum(to_string = "Dcy-")]
    Dcy(usize),
    #[strum(to_string = "Dcy2")]
    Dcy2(usize),
    #[strum(to_string = "Sus-")]
    Sus(usize),
    #[strum(to_string = "Rel-")]
    Rel(usize),
    #[strum(to_string = "Vol-")]
    Volume(f32),
}

impl Default for Sf2Cmd {
    fn default() -> Self {
        Self::Volume(1.0)
    }
}

#[derive(Clone, Copy, Default, Debug, States, PartialEq, Eq, Hash, Component)]
pub struct TrackID {
    pub id: usize,
    pub playing: bool,
}

#[derive(Clone, Copy, Default, Debug, States, PartialEq, Eq, Hash, Resource, Deref, DerefMut)]
pub struct FirstViewTrack(pub usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Event)]
pub enum CellCursorMoved {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Event)]
pub enum TracksScrolled {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Event)]
pub enum NoteChanged {
    SmallUp,
    SmallDown,
    BigUp,
    BigDown,
    Deleted,
}

pub fn display_midi_note(midi_note: MidiNote) -> String {
    let note_name_i = midi_note % 12;
    let octave = midi_note / 12;

    let note_names = [
        "C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-", "B#",
    ];
    let note_name = note_names[note_name_i as usize];

    format!("{note_name}{octave:X}")
}

pub fn playing(am_playing: Res<Playing>) -> bool {
    **am_playing
}

fn dpad_button_presed(button: GamepadButton, buttons: Res<ButtonPressTimers>) -> bool {
    if let Some((inst, just_pressed)) = buttons.0.get(&button) {
        return inst.elapsed() > Duration::from_secs_f64(0.25) || *just_pressed;
    }

    false
}

pub fn up_pressed(buttons: Res<ButtonPressTimers>) -> bool {
    dpad_button_presed(GamepadButton::DPadUp, buttons)
}

pub fn down_pressed(buttons: Res<ButtonPressTimers>) -> bool {
    dpad_button_presed(GamepadButton::DPadDown, buttons)
}

pub fn left_pressed(buttons: Res<ButtonPressTimers>) -> bool {
    dpad_button_presed(GamepadButton::DPadLeft, buttons)
}

pub fn right_pressed(buttons: Res<ButtonPressTimers>) -> bool {
    dpad_button_presed(GamepadButton::DPadRight, buttons)
}

pub fn a_and_b_pressed(inputs: Query<&Gamepad>) -> bool {
    for game_pad in inputs {
        if game_pad.pressed(GamepadButton::South) && game_pad.pressed(GamepadButton::East) {
            return true;
        }
    }

    false
}

pub fn north_button_pressed(inputs: Query<&Gamepad>) -> bool {
    for game_pad in inputs {
        if game_pad.just_pressed(GamepadButton::North) {
            return true;
        }
    }

    false
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         // let result = add(2, 2);
//         // assert_eq!(result, 4);
//     }
// }
