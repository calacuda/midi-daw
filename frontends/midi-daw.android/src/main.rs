#![feature(never_type)]
use crate::{
    playback::BASE_URL,
    tracks::{Track, TrackerCmd},
};
// use crossbeam::channel::{Receiver, Sender, unbounded};
use dioxus::prelude::*;
use midi_daw_types::{
    AddNoteBody, ChangeLenByBody, GetSequenceQuery, MidiChannel, RenameSequenceBody, RmNoteBody,
    Sequence, SetChannelBody, SetDevBody,
};
use std::{
    sync::{Arc, RwLock},
    thread::spawn,
};
use tracing::*;
use tungstenite::connect;

pub mod less_then;
pub mod playback;
pub mod tracks;

pub type SynthId = String;
pub type SectionsUID = usize;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const N_STEPS: usize = 16;
const COUNTER: GlobalSignal<f64> = Signal::global(|| 0.);

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

    // let drums = Track::new(
    //     Some("Drum-Track".into()),
    //     1,
    //     "Midi Through:0".into(),
    //     true,
    //     Some(16),
    // );
    // // drums.chan = MidiChannel::Ch10;
    // let mut melodic = Track::default();
    // melodic.dev = " Midi Through:0".into();
    // melodic.name = "Melodic-1".into();
    // let sections = Arc::new(RwLock::new(vec![melodic, drums]));
    let sections = Arc::new(RwLock::new(Vec::<Track>::default()));
    let displaying_uuid = Arc::new(RwLock::new(0usize));
    // let (send, sync_pulse) = unbounded::<f64>();

    // this is here to remind me of some technique, but what? google the docs for this function.
    // #[cfg(android)]
    // jni_sys::call_android_function();
    let client = reqwest::blocking::Client::new();
    let names: Option<Vec<String>> = client
        .get(format!("http://{BASE_URL}/sequence/names"))
        .send()
        .map(|res| res.json().ok())
        .ok()
        .flatten();
    let _jh = spawn({
        move || {
            sync_pulse_reader();
        }
    });

    if let Some(mut names) = names
        && !names.is_empty()
    {
        names.sort();

        for name in names {
            match client
                .get(format!("http://{BASE_URL}/sequence"))
                .query(&GetSequenceQuery::new(name.clone()))
                .send()
                .map(|res| res.json::<Sequence>())
            {
                Ok(Ok(json)) => {
                    let mut track = Track::default();
                    track.name = name.clone();

                    track.steps = json
                        .steps
                        .iter()
                        .map(|step| tracks::Step::from(step.clone()))
                        .collect();
                    track.dev = json.midi_dev.clone();
                    track.chan = json.channel.clone();
                    sections.write().unwrap().push(track);
                }
                Ok(Err(e)) => error!("invalid json returned from server. {e}"),
                Err(e) => error!("refreshing seqeunce failed with error, {e}"),
            }
        }
    } else {
        let track = Track::default();

        if let Err(e) = client
            .post(format!("http://{BASE_URL}/sequence/new"))
            .json(&track.name.clone())
            .send()
        {
            error!("adding track failed with error {e}");
        }

        sections.write().unwrap().push(track);
    }

    // dioxus::launch(App);
    dioxus::LaunchBuilder::new()
        .with_context(sections.clone())
        .with_context(displaying_uuid.clone())
        // .with_context(send)
        // .with_context(sync_pulse)
        // .with_context(send)
        .launch(App);
}

// fn sync_pulse_reader(tx: Sender<f64>) -> () {
fn sync_pulse_reader() -> () {
    // let bpq = 24;
    let (mut socket, response) = match connect(format!("ws://{BASE_URL}/message-bus")) {
        Ok(val) => val,
        Err(e) => {
            error!("{e}");
            return;
        }
    };

    if response.status() != 101 {
        error!(
            "failed to connect to message-bus. failure detected based on responce code. (was {}, expected 101.)",
            response.status()
        );
        return;
    }

    info!("connected... {}", response.status());

    loop {
        let Ok(msg) = socket.read() else {
            return;
        };

        match msg {
            tungstenite::Message::Binary(msg) if msg.len() == 8 => {
                let counter = f64::from_ne_bytes([
                    msg[0], msg[1], msg[2], msg[3], msg[4], msg[5], msg[6], msg[7],
                ]);
                info!("counter: {counter}");

                // if let Err(e) = tx.send(counter) {
                //     error!("{e}");
                // }
                COUNTER.set(counter);
            }
            _ => {}
        }
    }
}

#[component]
fn App() -> Element {
    let middle_view = use_signal(|| MiddleColView::Section);
    let sections = use_context::<Arc<RwLock<Vec<Track>>>>();
    let displaying_uuid = use_context::<Arc<RwLock<usize>>>();
    // let send = use_context::<Sender<f64>>();
    // let sync_pusle = use_context::<Receiver<f64>>();

    let sections = use_signal(|| sections);
    // let counter = use_signal(|| 0.0);
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
                LeftCol { middle_view, sections, displaying: displaying_uuid, edit_cell, playing_sections }
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
                                            let dev_name = dev_name.clone();

                                            async move {
                                                info!("setting the device to {dev_name}");

                                                sections.write().write().unwrap()[*displaying().read().unwrap()].dev = dev_name.clone();
                                                let track_name = sections.read().read().unwrap()[*displaying().read().unwrap()].name.clone();
                                                let client = reqwest::Client::new();

                                                if let Err(e) = client
                                                    .post(format!("http://{BASE_URL}/sequence/set-dev"))
                                                    .json(&SetDevBody::new(track_name, dev_name.clone()) )
                                                    .send().await
                                                {
                                                    error!("changing midi device failed with error {e}");
                                                }
                                            }
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
                                async move {
                                    info!("setting the midi channel to {channel:?}");
                                    sections.write().write().unwrap()[*displaying().read().unwrap()].chan = channel.clone();

                                    let track_name = sections.read().read().unwrap()[*displaying().read().unwrap()].name.clone();
                                    let client = reqwest::Client::new();

                                    if let Err(e) = client
                                        .post(format!("http://{BASE_URL}/sequence/set-channel"))
                                        .json(&SetChannelBody::new(track_name, channel.clone()) )
                                        .send().await
                                    {
                                        error!("setting channel failed with error {e}");
                                    }
                                }
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
                                async move {
                                    info!("setting the midi channel to {channel:?}");
                                    sections.write().write().unwrap()[*displaying().read().unwrap()].chan = channel.clone();

                                    let track_name = sections.read().read().unwrap()[*displaying().read().unwrap()].name.clone();
                                    let client = reqwest::Client::new();

                                    if let Err(e) = client
                                        .post(format!("http://{BASE_URL}/sequence/set-channel"))
                                        .json(&SetChannelBody::new(track_name, channel.clone()) )
                                        .send().await
                                    {
                                        error!("setting channel failed with error {e}");
                                    }
                                }
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

                        onclick: move |_| async move {
                            if let Some((row, cell)) = edit_cell() {
                                info!("{row} => {cell:?}");
                                let sections = sections.write();
                                let client = reqwest::Client::new();

                                match cell {
                                    Colums::Note => {
                                        // set note
                                        let track = &mut sections.write().unwrap()[*displaying().read().unwrap()];

                                        // connect to the API and rm the note
                                        for note in track.steps[row].note.iter() {
                                            if let Err(e) = client
                                                .post(format!("http://{BASE_URL}/sequence/rm-note"))
                                                .json(&RmNoteBody::new(track.name.clone(), row, *note))
                                                .send().await
                                            {
                                                error!("rming failed with error {e}");
                                                return;
                                            }
                                        }
                                        track.steps[row].note = vec![];

                                        // refresh track/sequence
                                        match client
                                            .get(format!("http://{BASE_URL}/sequence"))
                                            .query(&GetSequenceQuery::new(track.name.clone()))
                                            .send().await
                                        {
                                            Ok(res) => match res.json::<Sequence>().await {
                                                Ok(json) => {
                                                    track.steps = json.steps.iter().map(|step| tracks::Step::from(step.clone())).collect();
                                                    track.dev = json.midi_dev.clone();
                                                    track.chan = json.channel.clone();
                                                }
                                                Err(e) => error!("invalid json. {e}"),
                                            }
                                            Err(e) => error!("refreshing seqeunce failed with error, {e}"),
                                        }
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

                        onclick: move |_| async move {
                            if let Some((row, cell)) = edit_cell() {
                                info!("{row} => {cell:?}");
                                let sections = sections.write();
                                let client = reqwest::Client::new();

                                match cell {
                                    Colums::Note => {
                                        // set note
                                        // sections.write().unwrap()[*displaying().read().unwrap()].steps[row].note = vec![note()];

                                        // set velocity if not yet set
                                        if sections.read().unwrap()[*displaying().read().unwrap()].steps[row].velocity.is_none() {
                                            sections.write().unwrap()[*displaying().read().unwrap()].steps[row].velocity = Some(85);
                                        }
                                        // connect to the API and set the note with the velocity.
                                        let track = &mut sections.write().unwrap()[*displaying().read().unwrap()];

                                        for note in track.steps[row].note.iter() {
                                            if let Err(e) = client
                                                .post(format!("http://{BASE_URL}/sequence/rm-note"))
                                                .json(&RmNoteBody::new(track.name.clone(), row, *note))
                                                .send().await
                                            {
                                                error!("rming failed with error {e}");
                                                return;
                                            }
                                        }

                                        if let Err(e) = client
                                            .post(format!("http://{BASE_URL}/sequence/add-note"))
                                            .json(&AddNoteBody::new(track.name.clone(), row, note(), track.steps[row].velocity.unwrap_or(85), None))
                                            .send().await
                                        {
                                            error!("adding note failed with error {e}");
                                            return;
                                        }

                                        // refresh track/sequence
                                        match client
                                            .get(format!("http://{BASE_URL}/sequence"))
                                            .query(&GetSequenceQuery::new(track.name.clone()))
                                            .send().await
                                        {
                                            Ok(res) => match res.json::<Sequence>().await {
                                                Ok(json) => {
                                                    track.steps = json.steps.iter().map(|step| tracks::Step::from(step.clone())).collect();
                                                    track.dev = json.midi_dev.clone();
                                                    track.chan = json.channel.clone();
                                                }
                                                Err(e) => error!("invalid json. {e}"),
                                            }
                                            Err(e) => error!("refreshing seqeunce failed with error, {e}"),
                                        }
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

                // div {
                //     class: "drum-row",
                //
                //     for step_n in 0..sections.read().read().unwrap()[*displaying().read().unwrap()].steps.len() {
                //         div {
                //             "{step_n + 1}"
                //         }
                //     }
                // }

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
                                onclick: move |_| async move {
                                    let sections = sections.write();
                                    let track_name = sections.read().unwrap()[*displaying().read().unwrap()].name.clone();
                                    let step = &mut sections.write().unwrap()[*displaying().read().unwrap()].steps[step_n];
                                    let client = reqwest::Client::new();

                                    if step.note.contains(&drum_note) {
                                        step.note.retain(|elm| *elm != drum_note);
                                        // rm drum note
                                        if let Err(e) = client
                                            .post(format!("http://{BASE_URL}/sequence/rm-note"))
                                            .json(&RmNoteBody::new(track_name.clone(), step_n, drum_note))
                                            .send().await
                                        {
                                            error!("rming failed with error {e}");
                                            return;
                                        }
                                    } else {
                                        step.note.push(drum_note);
                                        // add drum note
                                        if let Err(e) = client
                                            .post(format!("http://{BASE_URL}/sequence/add-note"))
                                            .json(&AddNoteBody::new(track_name.clone(), step_n, drum_note, 99, None))
                                            .send().await
                                        {
                                            error!("adding note failed with error {e}");
                                            return;
                                        }
                                    }
                                },
                                "{step_n + 1:<2}"
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
                                class: {
                                    let mut class = "lin-number".into();

                                    if COUNTER() % (N_STEPS as f64) == i as f64 {
                                        class = format!("{class} text-red");
                                    }

                                    class
                                },
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
    playing_sections: Signal<Vec<usize>>,
) -> Element {
    // let com_mpsc = use_context::<Sender<MessageToPlayer>>();
    let mut listing = use_signal(|| MiddleColView::Section);
    let view_sections = || listing() == MiddleColView::Section;
    // info!("left-col");
    let mut editing_name = use_signal(|| None);
    let mut new_name = use_signal(|| String::new());

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

            for (uid, name) in match listing() {
                MiddleColView::Section => {
                    // sections().read().unwrap().iter().map(|section| (section.name.clone(), section.uuid)).collect::<Vec<_>>()
                    sections().read().unwrap().iter().map(|section| section.name.clone()).enumerate().collect::<Vec<_>>()
                }
                MiddleColView::Pattern => {
                    [].iter().map(|pattern: &String| pattern.to_owned()).enumerate().collect::<Vec<_>>()
                }
            } {
                div {
                    class: "row",

                    // edit-name button
                    div {
                        class: "button button-w-border super-center text-green",
                        onclick: {
                            let name = name.clone();

                            move |_| {
                                info!("editing the name of section {name}");
                                _ = editing_name.write().replace(uid);
                                *new_name.write() = name.clone();
                            }
                        },

                        "O"
                    }

                    if editing_name.read().is_some_and(|val| val == uid) {
                        form {
                            onsubmit: move |_| async move {
                                if let Ok(mut sections) = sections.write().write() {
                                    let client = reqwest::Client::new();
                                    // Rename track/seqeunce with API
                                    if let Err(e) = client
                                        .post(format!("http://{BASE_URL}/sequence/rename"))
                                        .json(&RenameSequenceBody::new(sections[uid].name.clone(), new_name.read().clone()))
                                        .send().await
                                    {
                                        error!("renaming failed with error {e}");
                                        return;
                                    }

                                    let track_name = sections[uid].name.clone();

                                    // get information about track/sequence
                                    match client
                                        .get(format!("http://{BASE_URL}/sequence"))
                                        .query(&GetSequenceQuery::new(track_name))
                                        .send().await
                                    {
                                        Ok(res) => match res.json::<Sequence>().await {
                                            Ok(json) => {
                                                sections[uid].steps = json.steps.iter().map(|step| tracks::Step::from(step.clone())).collect();
                                                sections[uid].dev = json.midi_dev.clone();
                                                sections[uid].chan = json.channel.clone();
                                            }
                                            Err(e) => error!("invalid json. {e}"),
                                        }
                                        Err(e) => error!("refreshing seqeunce failed with error, {e}"),
                                    }

                                    sections[uid].name = new_name.read().clone();

                                    *editing_name.write() = None;
                                }
                            },
                            input {
                                id: "rename-field",
                                name: "rename-field",
                                autofocus: "value",
                                r#type: "text",
                                value: "{new_name}",
                                onmounted: async move |cx| {
                                    if let Err(_e) = cx.set_focus(true).await {
                                        // error!("attempt to set focus to the rename section input field failed with: {e}")
                                    }
                                },
                                oninput: move |event| {
                                    new_name.set(event.value());
                                },
                            }
                        }
                    } else {
                        div {
                            id: {
                                if (listing() == middle_view()) && displaying().read().is_ok_and(|dis_uuid| uid == *dis_uuid ) {
                                    "displaying-sp".to_string()
                                } else {
                                    "".into()
                                }
                            },
                            class: "button nav-item",
                            onclick: move |_| async move {
                                middle_view.set(listing());
                                *displaying.write().write().unwrap() = uid;
                                edit_cell.set(None);
                                let client = reqwest::Client::new();
                                // let mut sections =  {
                                // let track = &mut sections.write().write().unwrap()[uid];
                                let track_name = sections.read().read().unwrap()[uid].name.clone();

                                // get information about track/sequence
                                    match client
                                        .get(format!("http://{BASE_URL}/sequence"))
                                        .query(&GetSequenceQuery::new(track_name))
                                        .send().await
                                    {
                                        Ok(res) => match res.json::<Sequence>().await {
                                            Ok(json) => {
                                                sections.write().write().unwrap()[uid].steps = json.steps.iter().map(|step| tracks::Step::from(step.clone())).collect();
                                                sections.write().write().unwrap()[uid].dev = json.midi_dev.clone();
                                                sections.write().write().unwrap()[uid].chan = json.channel.clone();
                                            }
                                            Err(e) => error!("invalid json. {e}"),
                                        }
                                        Err(e) => error!("refreshing seqeunce failed with error, {e}"),
                                    }
                            },
                            {name.clone()}
                        }
                    }

                    // delete section button
                    div {
                        class: "button button-w-border super-center text-red",
                        onclick: {
                            // let com_mpsc = com_mpsc.clone();

                            move |_| async move {
                                // if let Ok(mut sections) = sections.write() {
                                    let index_to_remove = uid;
                                    let dis = *displaying.read().read().unwrap();
                                    let track_name = sections.read().read().unwrap()[dis].name.clone();
                                    info!("removing section: {}", sections.read().read().unwrap()[index_to_remove].name);

                                    // must be "<=", if this is changed to "<" app will crash if
                                    // there are only two sections, a non-drum track and a drum
                                    // track, and the drum track is displayed then deleted.
                                    if index_to_remove <= dis && dis > 0 {
                                        *displaying.write().write().unwrap() -= 1;
                                    }

                                    if playing_sections.read().contains(&index_to_remove) {
                                        // _ = com_mpsc.send(MessageToPlayer::StopSection(index_to_remove));
                                        playing_sections.write().retain(|uid| *uid != index_to_remove);
                                        playing_sections.write().iter_mut().for_each(|i| if *i > index_to_remove {
                                            *i -= 1;
                                        });
                                    }

                                    // _ = com_mpsc.send(MessageToPlayer::DeletedSection(index_to_remove));
                                    let client = reqwest::Client::new();

                                    // connect to API and Delete,
                                    if let Err(e) = client
                                        .post(format!("http://{BASE_URL}/sequence/rm"))
                                        .json(&track_name)
                                        .send().await
                                    {
                                        error!("rming sequence failed with error {e}");
                                        return;
                                    }

                                    sections.write().write().unwrap().remove(index_to_remove);

                                    if sections.read().read().unwrap().is_empty() {
                                        sections.write().write().unwrap().push(Track::default());
                                    }
                                // }
                            }
                        },

                        "X"
                    }
                }
            }
        }

        hr {
            class: "full-width"
        }

        div {
            class: "full-width row",

            div {
                id: "add-section-or-pattern",
                class: "button button-w-border full-width super-center",
                onclick: move |_| async move {
                    if let Ok(mut sections) = sections.write().write() {
                        let uid = sections.len();
                        let name = format!("Section-{}", uid + 1);
                        info!("adding drum section: {name}");

                        // add track/sequence using API
                        let track = &mut sections[*displaying().read().unwrap()];
                        let client = reqwest::Client::new();

                        if let Err(e) = client
                            .post(format!("http://{BASE_URL}/sequence/new"))
                            .json(&name.clone())
                            .send().await
                        {
                            error!("adding sequence failed with error {e}");
                            return;
                        }

                        if let Err(e) = client
                            .post(format!("http://{BASE_URL}/sequence/set-channel"))
                            .json(&SetChannelBody::new(name.clone(), MidiChannel::Ch10))
                            .send().await
                        {
                            error!("setting sequence channel failed with error {e}");
                            return;
                        }

                        // refresh track/sequence
                        match client
                            .get(format!("http://{BASE_URL}/sequence"))
                            .query(&GetSequenceQuery::new(name.clone()))
                            .send().await
                        {
                            Ok(res) => match res.json::<Sequence>().await {
                                Ok(json) => {
                                    track.steps = json.steps.iter().map(|step| tracks::Step::from(step.clone())).collect();
                                    track.dev = json.midi_dev.clone();
                                    track.chan = json.channel.clone();
                                }
                                Err(e) => error!("invalid json. {e}"),
                            }
                            Err(e) => error!("refreshing seqeunce failed with error, {e}"),
                        }

                        sections.push(Track::new(
                            Some(name),
                            uid,
                            "Midi Through:0".into(),
                            true,
                            Some(16),
                        ));
                    }
                },

                "+Drums"
            }

            div {
                id: "add-section-or-pattern",
                class: "button button-w-border full-width super-center",
                onclick: move |_| async move {
                    if let Ok(mut sections) = sections.write().write() {
                        let uid = sections.len();
                        let name = format!("Section-{}", uid + 1);
                        info!("adding melody section: {name}");

                        // add track/sequence using API
                        let track = &mut sections[*displaying().read().unwrap()];
                        let client = reqwest::Client::new();

                        if let Err(e) = client
                            .post(format!("http://{BASE_URL}/sequence/new"))
                            .json(&name)
                            .send().await
                        {
                            error!("adding sequence failed with error {e}");
                            return;
                        }

                        // refresh track/sequence
                        match client
                            .get(format!("http://{BASE_URL}/sequence"))
                            .query(&GetSequenceQuery::new(track.name.clone()))
                            .send().await
                        {
                            Ok(res) => match res.json::<Sequence>().await {
                                Ok(json) => {
                                    track.steps = json.steps.iter().map(|step| tracks::Step::from(step.clone())).collect();
                                    track.dev = json.midi_dev.clone();
                                    track.chan = json.channel.clone();
                                }
                                Err(e) => error!("invalid json. {e}"),
                            }
                            Err(e) => error!("refreshing seqeunce failed with error, {e}"),
                        }

                        sections.push(Track::new(
                            Some(name),
                            uid,
                            "Midi Through:0".into(),
                            false,
                            Some(N_STEPS),
                        ));
                    }
                },

                "+Lead"
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
    // let com_mpsc = use_context::<Sender<MessageToPlayer>>();

    let mut tempo: Signal<f64> = use_signal(|| 0.0);
    let mut new_tempo = use_signal(|| tempo());
    use_future(move || async move {
        let client = reqwest::Client::new();

        match client.get(format!("http://{BASE_URL}/tempo")).send().await {
            Ok(res) => match res.json::<f64>().await {
                Ok(loc_tempo) => {
                    info!("got new tempo from the server: {loc_tempo}");
                    tempo.set(loc_tempo);
                    new_tempo.set(loc_tempo);
                }
                Err(e) => {
                    error!("parsing json tempo from server failed with error: {e}");
                    0.0;
                }
            },
            Err(e) => {
                error!("tempo change failed with error: {e}");
                0.0;
            }
        }
    });
    let mut editing_tempo = use_signal(|| false);

    let change_len = move |amt, verb| {
        async move {
            let client = reqwest::Client::new();
            let track_name = sections.read().read().unwrap()[*displaying.read().read().unwrap()]
                .name
                .clone();

            // TODO: send make sequence longer
            if let Err(e) = client
                .post(format!("http://{BASE_URL}/sequence/change-len-by"))
                .json(&ChangeLenByBody::new(track_name.clone(), amt))
                .send()
                .await
            {
                error!("tempo change failed with error {e}");
                return;
            } else {
                info!("length of track, {track_name}, has been {verb} by one");
            }

            // refresh track/sequence
            match client
                .get(format!("http://{BASE_URL}/sequence"))
                .query(&GetSequenceQuery::new(track_name))
                .send()
                .await
            {
                Ok(res) => match res.json::<Sequence>().await {
                    Ok(json) => {
                        sections.write().write().unwrap()[*displaying.read().read().unwrap()]
                            .steps = json
                            .steps
                            .iter()
                            .map(|step| tracks::Step::from(step.clone()))
                            .collect();
                        sections.write().write().unwrap()[*displaying.read().read().unwrap()].dev =
                            json.midi_dev.clone();
                        sections.write().write().unwrap()[*displaying.read().read().unwrap()]
                            .chan = json.channel.clone();
                    }
                    Err(e) => error!("invalid json. {e}"),
                },
                Err(e) => error!("refreshing seqeunce failed with error, {e}"),
            }
        }
    };

    rsx! {
        div {
            id: "right-main",
            class: "full-width full-height",

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

            br {}

            div {
                id: "play-section",
                class: "button button-w-border super-center full-width",
                onclick: move |_| async move {
                    let dis = displaying.read();
                    let dis = dis.read().unwrap();
                    let client = reqwest::Client::new();
                    let track_name = sections.read().read().unwrap()[*dis].name.clone();

                    if !playing_sections.read().contains(&dis) {
                        // start playback
                        playing_sections.write().push(*dis);

                        // connect to API and start playback for this track
                        if let Err(e) = client
                            .post(format!("http://{BASE_URL}/sequence/play-one"))
                            .json(&track_name)
                            .send().await
                        {
                            error!("playing sequence failed with error {e}");
                        }
                    } else {
                        // stop playback
                        playing_sections.write().retain(|elm| *elm != *dis);

                        // connect to API and stop playback for this track
                        if let Err(e) = client
                            .post(format!("http://{BASE_URL}/sequence/queue-stop"))
                            .json(&vec![track_name])
                            .send().await
                        {
                            error!("playing sequence failed with error {e}");
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
            br {}
            hr {}
            br {}
            div {
                id: "tempo",
                class: "button super-center full-width row",

                div {
                    class: "text-yellow",

                    "Tempo:"
                }

                if editing_tempo() {
                    form {
                        onsubmit: move |_| async move {
                            let client = reqwest::Client::new();
                            // Rename track/seqeunce with API
                            if let Err(e) = client
                                .post(format!("http://{BASE_URL}/tempo"))
                                .json(&new_tempo())
                                .send().await
                            {
                                error!("tempo change failed with error {e}");
                                return;
                            } else {
                                info!("tempo send to server: {new_tempo}");
                            }

                            match client
                                .get(format!("http://{BASE_URL}/tempo"))
                                .send().await
                            {
                                Ok(res) => match res.json::<f64>().await {
                                    Ok(loc_tempo) => {
                                        info!("got new tempo from the server: {loc_tempo}");
                                        tempo.set(loc_tempo);
                                    }
                                    Err(e) => {
                                        error!("parsing json tempo from server failed with error: {e}");
                                        return;
                                    }
                                }
                                Err(e) => {
                                    error!("tempo change failed with error: {e}");
                                    return;
                                }
                            }

                            editing_tempo.set(false);
                        },
                        input {
                            id: "rename-field",
                            name: "rename-field",
                            autofocus: "value",
                            r#type: "number",
                            min: "60.0",
                            step: "any",
                            value: "{new_tempo}",
                            onmounted: async move |cx| {
                                if let Err(_e) = cx.set_focus(true).await {
                                    // error!("attempt to set focus to the rename section input field failed with: {e}")
                                }
                            },
                            oninput: move |event| {
                                if let Ok(tempo) = event.value().parse::<f64>() {
                                    *new_tempo.write() = tempo;
                                }
                                // else {
                                //     error!("bad parse");
                                // }
                            },
                        }
                    }
                } else {
                    div {
                        class: "button button-w-border super-center full-width",
                        onclick: move |_| async move {
                            editing_tempo.set(true);
                            new_tempo.set(tempo());
                        },

                        "{tempo:.1}"
                    }
                }
            }
            br {}
            div {
                id: "track-size-change",
                class: "super-center full-width col",

                div {
                    id: "track-size-change-title",
                    class: "super-center full-width text-yellow",

                    "Track Len:"
                }

                div {
                    id: "track-size-change-buttons",
                    class: "super-center full-width row",

                    // button to make the track longer
                    div {
                        class: "button button-w-border super-center track-size-button",
                        onclick: move |_| {
                            change_len(1, "extened")
                        },

                        "Longer"
                    }
                    // button to make the track shorter
                    div {
                        class: "button button-w-border super-center track-size-button",
                        onclick: move |_| {
                            // send make sequence shorter
                            change_len(-1, "shortened")
                        },

                        "Shorter"
                    }
                }
            }
            br {}
            hr {}
            br {}
            div {
                div {
                    id: "track-size-change-title",
                    class: "super-center full-width text-yellow",

                    "Set Track Type:"
                }
                div {
                    class: "button button-w-border super-center",
                    onclick: move |_| async move {
                        let is_drum = sections.read().read().unwrap()[*displaying.read().read().unwrap()].is_drum;

                        sections.write().write().unwrap()[*displaying.read().read().unwrap()].is_drum = !is_drum;
                    },

                    if sections.read().read().unwrap()[*displaying.read().read().unwrap()].is_drum {
                        "melodic"
                    } else {
                        "drums"
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
        assert_eq!(display_midi_note(&60), "C-4");
    }
}
