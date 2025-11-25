#![feature(never_type)]
// use android_usbser::usb;
use crate::{
    playback::{BASE_URL, MessageToPlayer, playback},
    tracks::{Track, TrackerCmd},
};
use crossbeam::channel::{Sender, bounded, unbounded};
use dioxus::prelude::*;
use midi_daw_types::MidiChannel;
use std::{
    sync::{Arc, RwLock},
    thread::spawn,
};
use tracing::*;

// pub mod synth;
pub mod less_then;
pub mod playback;
pub mod tracks;

pub type SynthId = String;
pub type SectionsUID = usize;
// pub type InstrumentId = String;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
// const HEADER_SVG: Asset = asset!("/assets/header.svg");
// const N_STEPS: usize = 128;
const N_STEPS: usize = 16;

// lazy_static! {
//     pub static ref CBEAM_CHANNELS: (Sender<MidiMessage>, Receiver<MidiMessage>) = unbounded();
//     pub static ref MIDI_SEND: Sender<MidiMessage> = CBEAM_CHANNELS.0.clone();
//     pub static ref MIDI_RECV: Receiver<MidiMessage> = CBEAM_CHANNELS.1.clone();
// }

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum MiddleColView {
    Section,
    Pattern,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Colums {
    Note,
    Velocity,
    Cmd1,
    Cmd2,
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");

    // needed bc audio output will fail if its started too soon.
    // let synth = make_synth( );

    let drums = Track::new(
        Some("Drum-Track".into()),
        1,
        "Midi Through:0".into(),
        true,
        Some(16),
    );
    // drums.chan = MidiChannel::Ch10;
    let mut melodic = Track::default();
    melodic.dev = " Midi Through:0".into();
    melodic.name = "Melodic-1".into();
    let sections = Arc::new(RwLock::new(vec![melodic, drums]));
    let displaying_uuid = Arc::new(RwLock::new(0usize));
    let (send, recv) = unbounded();

    let _join_handle = spawn({
        let sections = sections.clone();

        move || playback(sections, recv)
    });

    // #[cfg(android)]
    // jni_sys::call_android_function();

    // dioxus::launch(App);
    dioxus::LaunchBuilder::new()
        .with_context(sections.clone())
        .with_context(displaying_uuid.clone())
        .with_context(send)
        .launch(App);
}

#[component]
fn App() -> Element {
    let middle_view = use_signal(|| MiddleColView::Section);
    let sections = use_context::<Arc<RwLock<Vec<Track>>>>();
    let displaying_uuid = use_context::<Arc<RwLock<usize>>>();

    let sections = use_signal(|| sections);
    let displaying_uuid = use_signal(|| displaying_uuid);
    let playing_sections = use_signal(|| Vec::default());

    // used to give context to the edit note/velocity/cmd-1/cmd-2
    let edit_cell = use_signal(|| None::<(usize, Colums)>);
    let choosing_device = use_signal(|| false);
    // let known_midi_devs: Signal<Arc<[String]>> = use_signal(|| Vec::new().into());
    let known_midi_devs: Signal<Vec<String>> = use_signal(|| Vec::new());

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        main {
            div {
                id: "left-col",
                LeftCol { middle_view, sections, displaying: displaying_uuid, edit_cell }
            }
            div {
                id: "middle-col",
                MiddleCol { middle_view, sections, displaying: displaying_uuid, edit_cell, choosing_device }

                if edit_cell.read().is_some() && middle_view() == MiddleColView::Section {
                    EditSectionMenu { sections, displaying: displaying_uuid, edit_cell }
                }

                if *choosing_device.read() && middle_view() == MiddleColView::Section {
                    MidiDevChooser { sections, displaying: displaying_uuid, choosing_device, known_midi_devs }
                }
            }
            div {
                id: "right-col",
                // PlayTone {  }
                RightCol { middle_view, sections, displaying: displaying_uuid, playing_sections, known_midi_devs, choosing_device }
            }
        }
    }
}

#[component]
fn MidiDevChooser(
    mut sections: Signal<Arc<RwLock<Vec<Track>>>>,
    displaying: Signal<Arc<RwLock<usize>>>,
    known_midi_devs: Signal<Vec<String>>,
    choosing_device: Signal<bool>,
) -> Element {
    info!("midi device chooser");
    let mut midi_devs = use_signal(|| None);
    let _coroutine_handle = use_coroutine(move |_: UnboundedReceiver<()>| async move {
        info!("get devs");
        let midi_url = format!("http://{BASE_URL}/midi");
        info!("{midi_url}");
        let client = reqwest::Client::new();

        if let Ok(req) = client.get(midi_url).send().await {
            info!("devs req {:?}", req);
            // Vec::new()

            if let Ok(dev_list) = req.json::<Vec<String>>().await {
                info!("devs {:?}", dev_list);
                // dev_list
                midi_devs.write().replace(Ok(dev_list));
            } else {
                error!("returned device list was expected to be json but failed to parse as such.");
                // Vec::new()
                midi_devs.write().replace(Err(
                    "Expected JSON from the API, but did not recieve JSON.".into(),
                ));
            }
        } else {
            error!("failed to make get request to get a list of devices.");
            // Vec::new()
            midi_devs.write().replace(Err(format!(
                "The GET request to retrieve the dev list failed with an error."
            )));
        };
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/loading-dots.css") }

        div {
            class: "sub-menu",

            div {
                class: "row",

                div {
                    class: "x-large midi-scroll-item super-center text-yellow",
                    "Available Devices:"
                }

                div {
                    class: "button midi-scroll-item super-center text-red",
                    onclick: move |_| {
                        *choosing_device.write() = false;
                    },

                    "Exit"
                }
            }

            hr {}

            div {
                id: "midi-scroll-list",

                div {
                    id: "midi-scroll-div",

                    {
                        match midi_devs.read().to_owned() {
                            Some(Ok(midi_devs)) => rsx! {
                                for dev_name in midi_devs {
                                    div {
                                        class: "button midi-scroll-item super-center",
                                        onclick: move |_| {
                                            info!("setting the device to {dev_name}");
                                            sections.write().write().unwrap()[*displaying().read().unwrap()].dev = dev_name.clone();
                                        },

                                        if dev_name.clone() == sections.read().read().unwrap()[*displaying().read().unwrap()].dev {
                                            "* {dev_name.clone()} *"
                                        } else {
                                            {dev_name.clone()}
                                        }
                                    }
                                }
                            },
                            Some(Err(message)) => rsx! {
                                div {
                                    class: "midi-scroll-item text-red",

                                    {message}
                                }
                            },
                            None => rsx! {
                                div {
                                    class: "midi-scroll-item text-green super-center",

                                    "LOADING"

                                    div {
                                        class: "loading-container",
                                        span { class: "dot" }
                                        span { class: "dot" }
                                        span { class: "dot" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div {
                class: "space-between",

                div {
                    class: "row space-around text-line padding-under",

                    for channel in [
                        MidiChannel::Ch1,
                        MidiChannel::Ch2,
                        MidiChannel::Ch3,
                        MidiChannel::Ch4,
                        MidiChannel::Ch5,
                        MidiChannel::Ch6,
                        MidiChannel::Ch7,
                        MidiChannel::Ch8,
                        MidiChannel::Ch9,
                    ] {
                        div {
                            class: {
                                let mut class = "button button-w-border super-center".to_string();

                                if sections.read().read().unwrap()[*displaying().read().unwrap()].chan == channel.clone() {
                                    class += " selected-channel";
                                }

                                class
                            },
                            onclick: move |_| {
                                info!("setting the midi channel to {channel:?}");
                                sections.write().write().unwrap()[*displaying().read().unwrap()].chan = channel.clone();
                            },

                            "{channel:?}"
                        }
                    }
                }

                div {
                    class: "row space-around text-line",

                    for channel in [
                        // MidiChannel::Ch9,
                        MidiChannel::Ch10,
                        MidiChannel::Ch11,
                        MidiChannel::Ch12,
                        MidiChannel::Ch13,
                        MidiChannel::Ch14,
                        MidiChannel::Ch15,
                        MidiChannel::Ch16
                    ] {
                        div {
                            class: {
                                let mut class = "button button-w-border super-center".to_string();

                                if sections.read().read().unwrap()[*displaying().read().unwrap()].chan == channel.clone() {
                                    class += " selected-channel";
                                }

                                class
                            },
                            onclick: move |_| {
                                info!("setting the midi channel to {channel:?}");
                                sections.write().write().unwrap()[*displaying().read().unwrap()].chan = channel.clone();
                            },

                            "{channel:?}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EditSectionMenu(
    sections: Signal<Arc<RwLock<Vec<Track>>>>,
    displaying: Signal<Arc<RwLock<usize>>>,
    edit_cell: Signal<Option<(usize, Colums)>>,
) -> Element {
    let note = use_signal(|| {
        if let Some((row, _cell)) = edit_cell() {
            *sections.read().read().unwrap()[*displaying().read().unwrap()].steps[row]
                .note
                .first()
                .unwrap_or(&12u8)
        } else {
            0u8
        }
    });
    let velocity = use_signal(|| 85u8);
    let cmd = use_signal(|| TrackerCmd::None);

    rsx! {
        div {
            id: "edit-menu",
            class: "col sub-menu",

            div {
                // id: "set-menu",
                class: "full-width fill-height",

                div {
                    class: "row super-center space-around full-height",

                    div {
                        class: "button text-yellow super-center full-width full-height",
                        onclick: move |_| {
                            edit_cell.set(None);
                        },

                        "ESC"
                    }

                    div {
                        class: "button text-red super-center full-width full-height",

                        onclick: move |_| {
                            if let Some((row, cell)) = edit_cell() {
                                info!("{row} => {cell:?}");
                                let sections = sections.write();

                                match cell {
                                    Colums::Note => {
                                        // set note
                                        sections.write().unwrap()[*displaying().read().unwrap()].steps[row].note = vec![];
                                    }
                                    Colums::Velocity => {
                                        // set velocity
                                        sections.write().unwrap()[*displaying().read().unwrap()].steps[row].velocity = None;
                                    }
                                    Colums::Cmd1 => {
                                        // set cmd
                                        sections.write().unwrap()[*displaying().read().unwrap()].steps[row].cmds.0 = TrackerCmd::None;
                                    }
                                    Colums::Cmd2 => {
                                        // set cmd
                                        sections.write().unwrap()[*displaying().read().unwrap()].steps[row].cmds.1 = TrackerCmd::None;
                                    }
                                }
                            }

                            edit_cell.set(None);
                        },

                        "DEL"
                    }

                    div {
                        class: "button text-green super-center full-width full-height",

                        onclick: move |_| {
                            if let Some((row, cell)) = edit_cell() {
                                info!("{row} => {cell:?}");
                                let sections = sections.write();

                                match cell {
                                    Colums::Note => {
                                        // set note
                                        sections.write().unwrap()[*displaying().read().unwrap()].steps[row].note = vec![note()];

                                        // set velocity if not yet set
                                        if sections.read().unwrap()[*displaying().read().unwrap()].steps[row].velocity.is_none() {
                                            sections.write().unwrap()[*displaying().read().unwrap()].steps[row].velocity = Some(85);
                                        }

                                        info!("set note to {:?}", sections.read().unwrap()[*displaying().read().unwrap()].steps[row].note);
                                    }
                                    Colums::Velocity => {
                                        // set velocity
                                        sections.write().unwrap()[*displaying().read().unwrap()].steps[row].velocity = Some(velocity())
                                    }
                                    Colums::Cmd1 => {
                                        // set cmd
                                        sections.write().unwrap()[*displaying().read().unwrap()].steps[row].cmds.0 = cmd();
                                    }
                                    Colums::Cmd2 => {
                                        // set cmd
                                        sections.write().unwrap()[*displaying().read().unwrap()].steps[row].cmds.1 = cmd();
                                    }
                                }
                            }

                            edit_cell.set(None);
                        },

                        "SET"
                    }
                }

                hr {
                    class: "full-width",
                }
            }

            if let Some((_row, cell)) = edit_cell() {
                match cell {
                    // Colums::Note if !sections.read()[displaying()].is_drum => rsx! { EditNote { note } },
                    // Colums::Note if sections.read()[displaying()].is_drum => rsx! { EditDrum { note } },
                    Colums::Note => rsx! {
                        // br {}
                        // div {
                        // }

                        div {
                            class: "h1 super-center",

                            {display_midi_note(&note())}
                        }

                        div {
                            id: "edit-notes",
                            class: "full-width space-evenly",
                            // h1 {
                            //     // class: "xx-large",
                            //     class: "super-center",
                            //     {display_midi_note(&note())}
                            // }

                            EditNote { note }
                        }
                    },
                    _ => { rsx! { } }
                }
            }
        }
    }
}

#[component]
fn EditNote(note: Signal<u8>) -> Element {
    // let original_note = display_midi_note(note());
    let mut octave = use_signal(|| (note() / 12) as i8);
    let mut name = use_signal(|| (note() % 12) as i8);
    let note_names = [
        "C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-",
    ];

    rsx! {
        div {
            class: "xx-large super-center",

            "Note:"
        }

        div {
            class: "row space-around",

            for (i, display_name) in note_names.iter().enumerate() {
                div {
                    class: "button button-w-border large",
                    onclick: move |_| {
                        name.set(i as i8);
                        note.set((name() + octave() * 12) as u8);
                    },
                    "{display_name}"
                }
            }
        }

        div {
            class: "xx-large super-center",

            "Octave:"
        }
        div {
            class: "row space-around",

            div {
                class: "button button-w-border large",
                onclick: move |_| {
                    octave.set(
                        (
                            if octave() > 1 {
                                octave() - 1
                            } else {
                                9
                            }
                        ) % 10
                    );
                    note.set((name() + octave() * 12) as u8);
                },
                "<-"
            }
            div {
                class: "large",
                "{octave.read()}"
            }
            div {
                class: "button button-w-border large",
                onclick: move |_| {
                    octave.set((octave() % 9) + 1);
                    note.set((name() + octave() * 12) as u8);
                },
                "->"
            }
        }
    }
}

#[component]
fn MiddleCol(
    middle_view: Signal<MiddleColView>,
    sections: Signal<Arc<RwLock<Vec<Track>>>>,
    displaying: Signal<Arc<RwLock<usize>>>,
    edit_cell: Signal<Option<(usize, Colums)>>,
    choosing_device: Signal<bool>,
) -> Element {
    // let is_drum_track = use_signal(|| sections.read()[displaying()].is_drum);

    rsx! {
        div {
            id: "middle-main",
            if middle_view() == MiddleColView::Section && !sections.read().read().unwrap()[*displaying().read().unwrap()].is_drum {
                SectionDisplay { middle_view, sections, displaying, edit_cell, choosing_device }
            } else if middle_view() == MiddleColView::Section && sections.read().read().unwrap()[*displaying().read().unwrap()].is_drum {
                DrumSectionDisplay { middle_view, sections, displaying }
            } else if middle_view() == MiddleColView::Pattern {}
        }
    }
}

#[component]
fn DrumSectionDisplay(
    middle_view: Signal<MiddleColView>,
    sections: Signal<Arc<RwLock<Vec<Track>>>>,
    displaying: Signal<Arc<RwLock<usize>>>,
) -> Element {
    // info!("DrumSelectionDisplay");

    rsx! {
        div {
            id: "drum-section",

            div {
                id: "drum-labels",

                for (_, drum_name) in [
                    (36u8, "Kick"),
                    (40, "Snare"),
                    (45, "Low Tom"),
                    (50, "High Tom"),
                    (39, "Clap"),
                    (46, "Open HH"),
                    (42, "Closed HH")
                ] {
                    div {
                        class: "drum-label",

                        "{drum_name}"
                    }
                }
            }

            div {
                id: "drum-grid",

                for (drum_note, _) in [
                    (36u8, "Kick"),
                    (40, "Snare"),
                    (45, "Low Tom"),
                    (50, "High Tom"),
                    (39, "Clap"),
                    (46, "Open HH"),
                    (42, "Closed HH")
                ] {
                    div {
                        class: "drum-row",

                        for step_n in 0..sections.read().read().unwrap()[*displaying().read().unwrap()].steps.len() {
                            div {
                                class: {
                                    let mut classes = vec!["drum-button"];

                                    if sections.read().read().unwrap()[*displaying().read().unwrap()].steps[step_n].note.contains(&drum_note) {
                                        classes.push("drum-button-active");
                                    }

                                    classes.join(" ")
                                },
                                onclick: move |_| {
                                    let sections = sections.write();
                                    let step = &mut sections.write().unwrap()[*displaying().read().unwrap()].steps[step_n];

                                    if step.note.contains(&drum_note) {
                                        step.note.retain(|elm| *elm != drum_note);
                                    } else {
                                        step.note.push(drum_note);
                                    }
                                },
                                " "
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SectionDisplay(
    middle_view: Signal<MiddleColView>,
    sections: Signal<Arc<RwLock<Vec<Track>>>>,
    displaying: Signal<Arc<RwLock<usize>>>,
    edit_cell: Signal<Option<(usize, Colums)>>,
    choosing_device: Signal<bool>,
) -> Element {
    // info!("regular section view");

    rsx! {
        div {
            id: "section-display-header",
            div { "Line" }
            div { "Note" }
            div { "Vel" }
            div { "Cmd1" }
            div { "Cmd2" }
        }

        div {
            id: "section-scroll-list",

            div {
                id: "section-scroll-div",

                for (i, step) in sections().read().unwrap()[*displaying().read().unwrap()].steps.iter().enumerate() {
                    div {
                        class: "section-scroll-item",

                        div {
                            class: "section-row",
                            id: {
                                if i % 2 == 0 {
                                    "row-light"
                                } else {
                                    "row-dark"
                                }
                            },

                            // Line Number
                            div {
                                class: "lin-number",
                                // "{i:->2X}"
                                "{i + 1:->3}"
                            }
                            // Note
                            div {
                                onclick: move |_| {
                                    // open edit menu with context
                                    if edit_cell.read().is_none() {
                                        edit_cell.set(Some((i, Colums::Note)));
                                    }

                                    *choosing_device.write() = false;
                                },
                                class: "button super-center",

                                {step.note.first().map( display_midi_note ).unwrap_or("---".into())}
                            }
                            // Velocity
                            div {
                                onclick: move |_| {
                                    // open edit menu with context
                                    if edit_cell.read().is_none() {
                                        edit_cell.set(Some((i, Colums::Velocity)));
                                    }

                                    *choosing_device.write() = false;
                                },
                                class: "button super-center",

                                if sections().read().unwrap()[*displaying().read().unwrap()].steps[i].note.first().is_some() {
                                    // "{step.velocity.unwrap_or(85):->3X}"
                                    "{step.velocity.unwrap_or(85):->3}"
                                } else {
                                    "---"
                                }
                            }
                            // CMD 1
                            div {
                                onclick: move |_| {
                                    // open edit menu with context
                                    if edit_cell.read().is_none() {
                                        edit_cell.set(Some((i, Colums::Cmd1)));
                                    }

                                    *choosing_device.write() = false;
                                },
                                class: "button super-center",

                                "{step.cmds.0}"
                            }
                            // CMD 2
                            div {
                                onclick: move |_| {
                                    // open edit menu with context
                                    if edit_cell.read().is_none() {
                                        edit_cell.set(Some((i, Colums::Cmd2)));
                                    }

                                    *choosing_device.write() = false;
                                },
                                class: "button super-center",

                                "{step.cmds.1}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn LeftCol(
    middle_view: Signal<MiddleColView>,
    sections: Signal<Arc<RwLock<Vec<Track>>>>,
    displaying: Signal<Arc<RwLock<usize>>>,
    edit_cell: Signal<Option<(usize, Colums)>>,
) -> Element {
    let mut listing = use_signal(|| MiddleColView::Section);
    let view_sections = || listing() == MiddleColView::Section;

    info!("left-col");

    rsx! {
        div {
            id: "section-pattern-select",

            div {
                class: "button col normal-text",
                onclick: move |_| listing.set(MiddleColView::Section),

                div { "Section" }
                div {
                    class: {
                        let mut classes = vec!["led"];
                        if view_sections() { classes.push("led-on") }
                        classes.join(" ")
                    },
                }
            }
            div {
                class: "button col normal-text",
                onclick: move |_| listing.set(MiddleColView::Pattern),

                div { "Pattern" }
                div {
                    class: {
                        let mut classes = vec!["led"];

                        if !view_sections() { classes.push("led-on") }

                        classes.join(" ")
                    },
                }
            }
        }

        div {
            id: "nav-list",

            for (name, uuid) in match listing() {
                MiddleColView::Section => {
                    sections().read().unwrap().iter().map(|section| (section.name.clone(), section.uuid)).collect::<Vec<_>>()
                }
                MiddleColView::Pattern => {
                    [].iter().map(|pattern: &(String, usize)| pattern.to_owned()).collect::<Vec<_>>()
                }
            } {
                // TODO: add edit-name button here
                div {
                    id: {
                        if (listing() == middle_view()) && displaying().read().is_ok_and(|dis_uuid| uuid == *dis_uuid ) {
                            "displaying-sp".to_string()
                        } else {
                            "".into()
                        }
                    },
                    class: "button nav-item",
                    onclick: move |_| {
                        middle_view.set(listing());
                        *displaying.write().write().unwrap() = uuid;
                        edit_cell.set(None);
                    },
                    {name}
                }
                // TODO: add delete section button here
            }
        }

        hr {
            class: "full-width"
        }

        div {
            class: "full-width row",

            div {
                id: "add-section-or-pattern",
                class: "button button-w-border full-width",
                onclick: move |_| {
                    if let Ok(mut sections) = sections.write().write() {
                        let uid = sections.len();
                        let name = format!("Section-{}", uid + 1);
                        info!("adding section: {name}");

                        sections.push(Track::new(
                            Some(name),
                            uid,
                            "Midi Through:0".into(),
                            true,
                            Some(16),
                        ));
                    }
                },

                "+drums"
            }

            div {
                id: "add-section-or-pattern",
                class: "button button-w-border full-width",
                onclick: move |_| {
                    if let Ok(mut sections) = sections.write().write() {
                        let uid = sections.len();
                        let name = format!("Section-{}", uid + 1);
                        info!("adding section: {name}");

                        sections.push(Track::new(
                            Some(name),
                            uid,
                            "Midi Through:0".into(),
                            false,
                            Some(N_STEPS),
                        ));
                    }
                },

                "+lead"
            }
        }
    }
}

#[component]
fn RightCol(
    middle_view: Signal<MiddleColView>,
    sections: Signal<Arc<RwLock<Vec<Track>>>>,
    displaying: Signal<Arc<RwLock<usize>>>,
    // edit_cell: Signal<Option<(usize, Colums)>>,
    playing_sections: Signal<Vec<usize>>,
    known_midi_devs: Signal<Vec<String>>,
    choosing_device: Signal<bool>,
) -> Element {
    let com_mpsc = use_context::<Sender<MessageToPlayer>>();

    rsx! {
        div {
            id: "right-main",
            class: "full-width",

            // a button to set the device for the selected track
            div {
                id: "set-device",
                // TODO: gray out the button when middle_view() != MiddleColView::Section
                class: "button button-w-border super-center full-width",
                onclick: move |_| {
                    // get_devs.call();
                    let val = !*choosing_device.read();

                    *choosing_device.write() = val;
                },

                div {
                    class: "text-yellow",

                    "Midi Dev:"
                }

                hr {
                    class: "full-width",
                }

                div {
                    class: "large",

                    {sections.read().read().unwrap()[*displaying.read().read().unwrap()].dev.clone()}
                }
            }

            div {
                id: "play-section",
                class: "button button-w-border super-center",
                onclick: move |_| {
                    let dis = displaying.read();
                    let dis = dis.read().unwrap();

                    if !playing_sections.read().contains(&dis) {
                        // start playback
                        playing_sections.write().push(*dis);

                        if let Err(e) = com_mpsc.send(MessageToPlayer::PlaySection(*dis)) {
                            error!("attempting to send start playback message failed with error: {e}");
                        }
                    } else {
                        // stop playback
                        playing_sections.write().retain(|elm| *elm != *dis);

                        if let Err(e) = com_mpsc.send(MessageToPlayer::StopSection(*dis)) {
                            error!("attempting to send stop playback message failed with error: {e}");
                        }
                    }
                },

                if !playing_sections.read().contains(&displaying.read().read().unwrap()) {
                    div {
                        class: "row",

                        "Play"

                        div {
                            class: "text-green",

                            "|>"
                        }
                    }
                } else {
                    div {
                        class: "row",

                        "Stop"

                        div {
                            class: "text-red",

                            "[]"
                        }
                    }
                }
            }
        }
    }
}

pub fn display_midi_note(midi_note: &u8) -> String {
    let note_name_i = midi_note % 12;
    let octave = midi_note / 12;

    let note_names = [
        "C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-",
    ];
    let note_name = note_names[note_name_i as usize];

    format!("{note_name}{octave:X}")
}

pub fn display_midi_drum(midi_note: u8) -> String {
    let note_name_i = midi_note % 12;
    let octave = midi_note / 12;

    let note_names = [
        "C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-",
    ];
    let note_name = note_names[note_name_i as usize];

    format!("{note_name}{octave:X}")
}

// #[component]
// fn PlayTone() -> Element {
//     let mut playing = false;
//
//     rsx! {
//         button { onclick: move |_| {
//             let send = if !playing {
//                 // MIDI_SEND.send(MidiMessage::NoteOn(midi_control::Channel::Ch1, KeyEvent { key: 48, value: 90 }))
//             } else {
//                 // MIDI_SEND.send(MidiMessage::NoteOff(midi_control::Channel::Ch1, KeyEvent { key: 48, value: 90 }))
//             };
//
//             if let Err(e) = send {
//                 error!("{e}");
//             } else {
//                 playing = !playing;
//                 info!("playing: {playing}");
//             }
//         }, "Play Example Tone"  }
//     }
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn note_display() {
        assert_eq!(display_midi_note(60), "C-4");
    }
}
