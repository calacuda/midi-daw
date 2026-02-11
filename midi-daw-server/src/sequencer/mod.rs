use std::time::Duration;

use actix::dev::OneshotSender;
use async_std::{
    fs::{File, create_dir_all, read_dir, remove_file},
    io::{BufReadExt, BufReader, ReadExt, WriteExt},
};
use crossbeam::channel::Receiver;
use futures_lite::stream::StreamExt;
use fx_hash::FxHashMap;
use http_body_util::Full;
use hyper::{Method, Request, body::Bytes};
use hyper_util::client::legacy::Client;
use hyperlocal::{UnixClientExt, UnixConnector, Uri};
use midi_daw_types::{
    BPQ, MidiChannel, MidiMsg, MidiReqBody, NoteDuration, Sequence, SequenceName, Tempo,
    UDS_SERVER_PATH,
};
use tokio::spawn;
use tracing::*;
use xdg::BaseDirectories;

use crate::{midi::out::unwrap_rw_lock, server::message_bus::MbServerHandle};

pub type AllSequences = FxHashMap<SequenceName, Sequence>;

// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SequencerControlCmd {
    GetSequence {
        sequence: SequenceName,
        responder: OneshotSender<Option<Sequence>>,
    },
    GetSequences {
        responder: OneshotSender<Vec<String>>,
    },
    NewSequence {
        name: Option<SequenceName>,
        midi_dev: Option<String>,
        channel: Option<MidiChannel>,
        // for allowing sequences of different sizes
        // len: usize,
    },
    SetSequenceDev {
        name: SequenceName,
        midi_dev: String,
    },
    SetSequenceChannel {
        name: SequenceName,
        channel: MidiChannel,
    },
    RenameSequence {
        old_name: SequenceName,
        new_name: SequenceName,
    },
    RmSequence {
        name: SequenceName,
    },
    Play(Vec<SequenceName>),
    PlayAll,
    Stop(Vec<SequenceName>),
    StopAll,
    QueueStop(Vec<SequenceName>),
    Pause(Vec<SequenceName>),
    PauseAll,
    AddNote {
        sequence: SequenceName,
        step: usize,
        note: u8,
        velocity: u8,
        note_len: Option<NoteDuration>,
    },
    RmNote {
        sequence: SequenceName,
        step: usize,
        note: u8,
        // note_len: Option<NoteDuration>,
    },
    AddCmd {
        sequence: SequenceName,
        step: usize,
        cmd: MidiMsg,
    },
    RmCmd {
        sequence: SequenceName,
        step: usize,
        cmd: MidiMsg,
    },
    ChangeLenBy {
        sequence: SequenceName,
        amt: isize,
    },
    /// saves a sequence to disk
    SaveSequence {
        /// the sequence name to save
        sequence: SequenceName,
    },
    /// lists sequences that have been saved to disk that are not a part of a project
    ListSavedSequences {
        /// will send back the base file names without the parent directory
        responder: OneshotSender<Vec<String>>,
    },
    /// loads a sequence from disk
    LoadSequence {
        /// the sequence name to load
        sequence: SequenceName,
    },
    ///
    RmSavedSequence {
        /// Sequence name to rm
        sequence: SequenceName,
    },
    /// saves all seqeunces into a sub folder of the data dir. the base file name will be based on the sequences name
    SaveProject {
        project_name: String,
    },
    /// lists only the projects that have been saved (not their sequences)
    ListSavedProjects {
        /// will send back the base file names without the parent directory
        responder: OneshotSender<Vec<String>>,
    },
    /// loads a Project and its sequences from disk
    LoadSavedProject {
        /// the sequence name to load
        project_name: String,
    },
    /// rm a project from storage.
    RmSavedProject {
        /// Sequence name to rm
        project_name: String,
    },
}

#[tokio::main]
pub async fn sequencer_start(
    tempo: Tempo,
    bpq: BPQ,
    controls: Receiver<SequencerControlCmd>,
    mb_sender: MbServerHandle,
) {
    // let url = Uri::new("/tmp/hyperlocal.sock", "/").into();

    let mk_timer = || {
        // calculate sleep_time based on BPQ & tempo.
        let (tempo, beats) = (unwrap_rw_lock(&tempo, 99.), unwrap_rw_lock(&bpq, 24.));
        let sleep_time = Duration::from_secs_f64((60.0 / tempo) / beats);

        move || {
            std::thread::sleep(sleep_time);
        }
    };

    // start a sleep thread to sleep for sleep_time.
    let mut sleep_thread = std::thread::spawn(mk_timer());
    let mut counter = 0.;

    let mut sequences: AllSequences = FxHashMap::default();
    let mut queued_sequences: Vec<SequenceName> = Vec::default();
    let mut queued_stop_sequences: Vec<SequenceName> = Vec::default();
    let mut playing_sequences: Vec<SequenceName> = Vec::default();
    let mut jh_s = Vec::default();
    let conn = uuid::Uuid::new_v4();

    loop {
        if sleep_thread.is_finished() {
            sleep_thread = std::thread::spawn(mk_timer());

            if (counter % (unwrap_rw_lock(&bpq, 24.) / 4.)) == 0.0 {
                let i = counter / (unwrap_rw_lock(&bpq, 24.) / 4.);
                // info!("i = {i}");
                // info!("i % 16 = {}", i as usize % 16);

                if i % 16. == 0. || playing_sequences.is_empty() {
                    playing_sequences.append(&mut queued_sequences);
                }

                playing_sequences.retain(|name| {
                    if let Some(sequence) = sequences.get(name) {
                        if i as usize % sequence.steps.len() == 0 {
                            let res = !queued_stop_sequences.contains(name);
                            // queued_stop_sequences.retain(|stop_name| stop_name != name);
                            res
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                });
                queued_stop_sequences.retain(|stop_name| playing_sequences.contains(stop_name));

                // send midi messages from playing sequences
                let play_messages: Vec<MidiReqBody> = playing_sequences
                    .iter()
                    .filter_map(|name| {
                        if let Some(sequence) = sequences.get(name) {
                            debug!(
                                "sequence, {}, has {} steps",
                                sequence.name,
                                sequence.steps.len()
                            );

                            let msgs = sequence.steps[i as usize % sequence.steps.len()]
                                .iter()
                                .map(|msg| {
                                    MidiReqBody::new(
                                        sequence.midi_dev.clone(),
                                        sequence.channel,
                                        msg.clone(),
                                    )
                                });

                            if msgs.len() > 0 { Some(msgs) } else { None }
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .collect();

                if !play_messages.is_empty() {
                    let jh = spawn(async move {
                        trace!("playing {} notes.", play_messages.len());
                        let url = Uri::new(UDS_SERVER_PATH, "/batch-midi");
                        let client: Client<UnixConnector, Full<Bytes>> = Client::unix();
                        let req = Request::builder()
                            .method(Method::POST)
                            .uri(url)
                            .header("content-type", "application/json")
                            .body(Full::new(Bytes::from(
                                serde_json::json!(play_messages).to_string(),
                            )))
                            .unwrap();

                        match client.request(req).await {
                            Ok(_res) => {}
                            Err(e) => error!("failed to play a note, got error: {e}"),
                        }
                    });

                    jh_s.push(jh);

                    jh_s.retain(|jh| !jh.is_finished());
                    // break;
                }

                if !(playing_sequences.is_empty() && queued_sequences.is_empty()) {
                    mb_sender.send_binary(conn, i.to_ne_bytes().to_vec().into());
                }
            }

            if !(playing_sequences.is_empty() && queued_sequences.is_empty()) {
                counter += 1.;
                counter %= f64::MAX;
            } else {
                counter = 0.;
            }
        } else {
            while let Ok(msg) = controls.try_recv() {
                // do msg thing

                match msg {
                    SequencerControlCmd::GetSequence {
                        sequence,
                        responder,
                    } => {
                        if let Err(e) =
                            responder.send(sequences.get(&sequence).map(|seq| seq.clone()))
                        {
                            error!("sending sequence failed with error: {e:?}");
                        }
                    }
                    SequencerControlCmd::GetSequences { responder } => {
                        if let Err(e) =
                            responder.send(sequences.keys().map(|name| name.clone()).collect())
                        {
                            error!("sending sequence names failed with error: {e:?}");
                        }
                    }
                    SequencerControlCmd::NewSequence {
                        name,
                        midi_dev,
                        channel,
                    } => {
                        let mut seq = Sequence::default();

                        if let Some(name) = name {
                            seq.name = name;
                        }

                        if let Some(midi_dev) = midi_dev {
                            seq.midi_dev = midi_dev;
                        }

                        if let Some(channel) = channel {
                            seq.channel = channel;
                        }

                        sequences.insert(seq.name.clone(), seq);
                    }
                    SequencerControlCmd::SetSequenceDev { name, midi_dev } => {
                        if let Some(seq) = sequences.get_mut(&name) {
                            seq.midi_dev = midi_dev;
                        }
                    }
                    SequencerControlCmd::SetSequenceChannel { name, channel } => {
                        if let Some(seq) = sequences.get_mut(&name) {
                            seq.channel = channel;
                        }
                    }
                    SequencerControlCmd::RenameSequence { old_name, new_name } => {
                        if let Some(mut seq) = sequences.remove(&old_name) {
                            seq.name = new_name.clone();

                            if playing_sequences.contains(&old_name) {
                                playing_sequences.retain(|name| name.clone() != old_name);
                                playing_sequences.push(new_name.clone());
                            }

                            if queued_sequences.contains(&old_name) {
                                queued_sequences.retain(|name| name.clone() != old_name);
                                queued_sequences.push(new_name.clone());
                            }

                            sequences.insert(new_name, seq);
                        }
                    }
                    SequencerControlCmd::RmSequence { name } => {
                        sequences.remove(&name);
                        queued_sequences.retain(|n| n != &name);
                        playing_sequences.retain(|n| n != &name);
                        queued_stop_sequences.retain(|stop_name| stop_name != &name);

                        if sequences.is_empty()
                            || (playing_sequences.is_empty() && queued_sequences.is_empty())
                        {
                            counter = 0.;
                        }
                    }
                    SequencerControlCmd::Play(names) => names.into_iter().for_each(|name| {
                        queued_stop_sequences.retain(|stop_name| stop_name != &name);

                        if sequences.contains_key(&name) {
                            queued_sequences.push(name);
                        } else {
                            error!("unknown sequence, \"{name}\"");
                        }
                    }),
                    SequencerControlCmd::PlayAll => {
                        sequences
                            .keys()
                            .for_each(|name| queued_sequences.push(name.clone()));
                    }
                    SequencerControlCmd::Stop(names) => {
                        queued_sequences.retain(|name| !names.contains(name));
                        playing_sequences.retain(|name| !names.contains(name));

                        if playing_sequences.is_empty() && queued_sequences.is_empty() {
                            counter = 0.;
                        }
                    }
                    SequencerControlCmd::StopAll => {
                        queued_sequences.clear();
                        playing_sequences.clear();
                        counter = 0.;
                    }
                    SequencerControlCmd::QueueStop(names) => {
                        queued_stop_sequences.append(&mut names.clone());
                    }
                    SequencerControlCmd::Pause(_names) => warn!("not implemented yet"),
                    SequencerControlCmd::PauseAll => warn!("not implemented yet"),
                    SequencerControlCmd::AddNote {
                        sequence,
                        step: step_i,
                        note,
                        velocity,
                        note_len,
                    } => {
                        if let Some(seq) = sequences.get_mut(&sequence) {
                            if let Some(step) = seq.steps.get_mut(step_i) {
                                step.push(MidiMsg::PlayNote {
                                    note,
                                    velocity,
                                    duration: note_len.unwrap_or(NoteDuration::Sn(1)),
                                });
                            } else {
                                error!(
                                    "invalid step, {step_i}. sequence, \"{sequence}\", only has {}, steps",
                                    seq.steps.len()
                                );
                            }
                        } else {
                            error!("sequence not found");
                        }
                    }
                    SequencerControlCmd::RmNote {
                        sequence,
                        step: step_i,
                        note,
                    } => {
                        if let Some(seq) = sequences.get_mut(&sequence) {
                            if let Some(step) = seq.steps.get_mut(step_i) {
                                step.retain(|msg| {
                                    let MidiMsg::PlayNote {
                                        note: msg_note,
                                        velocity: _,
                                        duration: _,
                                    } = msg
                                    else {
                                        return true;
                                    };

                                    *msg_note != note
                                });
                            } else {
                                error!(
                                    "invalid step, {step_i}. sequence, \"{sequence}\", only has {}, steps",
                                    seq.steps.len()
                                );
                            }
                        } else {
                            error!("sequence not found");
                        }
                    }
                    SequencerControlCmd::AddCmd {
                        sequence: _,
                        step: _,
                        cmd: _,
                    } => warn!("not implemented yet"),
                    SequencerControlCmd::RmCmd {
                        sequence: _,
                        step: _,
                        cmd: _,
                    } => warn!("not implemented yet"),
                    SequencerControlCmd::ChangeLenBy { sequence, amt } => {
                        if let Some(seq) = sequences.get_mut(&sequence) {
                            if amt > 0 {
                                (0..amt).for_each(|_| seq.steps.push(Vec::default()));
                            } else if amt < 0 && (seq.steps.len() + amt as usize) > 0 {
                                (0..amt.abs()).for_each(|_| {
                                    seq.steps.pop();
                                });
                            }
                        } else {
                            error!("sequence not found");
                        }
                    }
                    SequencerControlCmd::SaveSequence { sequence } => {
                        if let Some(seq) = sequences.get(&sequence) {
                            save_seqeunce(seq, &sequence).await;
                            // logging handled in above function
                        } else {
                            error!("sequence not found");
                        }
                    }
                    SequencerControlCmd::ListSavedSequences { responder } => {
                        let xdg_dirs = BaseDirectories::new();

                        if let Some(mut data_dir) = xdg_dirs.data_home {
                            data_dir.push("midi-daw");

                            if let Ok(mut dir_contents) = read_dir(data_dir).await {
                                let mut contents = Vec::default();

                                while let Some(f_name) = dir_contents.next().await {
                                    if let Ok(fname) = f_name {
                                        if let Ok(ftype) = fname.file_type().await
                                            && ftype.is_file()
                                        {
                                            contents.push(
                                                fname.file_name().to_string_lossy().to_string(),
                                            )
                                        }
                                    }
                                }

                                if let Err(e) = responder.send(contents.clone()) {
                                    error!(
                                        "attempts to respond to front end failed with error: {e:?}"
                                    );
                                } else {
                                    info!("listed seqeunces {:?}", contents);
                                }
                            } else {
                                error!("failed to read saved files in data directory.");
                            }
                        } else {
                            error!(
                                "the '$HOME' env var could not be found. so no xdg dir could be set"
                            );
                        }
                    }
                    SequencerControlCmd::LoadSequence { sequence } => {
                        // get file path
                        let xdg_dirs = BaseDirectories::new();

                        if let Some(mut data_dir) = xdg_dirs.data_home {
                            data_dir.push("midi-daw");
                            data_dir.push(format!("{}.json", sequence));

                            // read file from disk
                            if let Ok(file) = File::open(&data_dir).await {
                                let mut reader = BufReader::new(file);

                                let mut json_text = String::new();
                                if reader.read_to_string(&mut json_text).await.is_ok() {
                                    // let mut last_f_len = json_text.len();
                                    // reader.read_line(&mut json_text);
                                    // let mut f_len = json_text.len();
                                    //
                                    // while last_f_len != f_len {
                                    //     last_f_len = f_len;
                                    //     reader.read_line(&mut json_text);
                                    //     f_len = json_text.len();
                                    // }

                                    // parse JSON
                                    if let Ok(json) = serde_json::from_str::<Sequence>(&json_text) {
                                        sequences.insert(json.name.clone(), json.clone());
                                        info!("restored sequnce, '{}', from disk", json.name);
                                    } else {
                                        error!("parsing stored json failed.");
                                    }
                                } else {
                                    error!("reading file failed.");
                                }
                            } else {
                                error!(
                                    "failed to open file. does, '{}', exist?",
                                    data_dir.to_string_lossy()
                                );
                            }
                        } else {
                            error!(
                                "the '$HOME' env var could not be found. so no xdg dir could be set"
                            );
                        }
                    }
                    SequencerControlCmd::RmSavedSequence { sequence } => {
                        // get file path
                        let xdg_dirs = BaseDirectories::new();

                        if let Some(mut data_dir) = xdg_dirs.data_home {
                            data_dir.push("midi-daw");

                            if !sequence.ends_with(".json") {
                                data_dir.push(format!("{}.json", sequence));
                            } else {
                                data_dir.push(sequence);
                            }

                            if data_dir.exists() {
                                if let Err(e) = remove_file(&data_dir).await {
                                    error!(
                                        "removing, '{}', failed with error, {e}",
                                        data_dir.to_string_lossy()
                                    );
                                } else {
                                    info!("seqeunce file removed");
                                }
                            } else {
                                warn!(
                                    "the sequence file path: '{}' doesn't exist, can't remove",
                                    data_dir.to_string_lossy()
                                );
                            }
                        } else {
                            error!(
                                "the '$HOME' env var could not be found. so no xdg dir could be set"
                            );
                        }
                    }
                    SequencerControlCmd::SaveProject { project_name } => {
                        match serde_json::to_string(&sequences) {
                            Ok(json) => {
                                let xdg_dirs = BaseDirectories::new();

                                if let Some(mut data_dir) = xdg_dirs.data_home {
                                    data_dir.push("midi-daw");
                                    data_dir.push("projects");

                                    if !data_dir.exists() {
                                        if let Err(e) = create_dir_all(&data_dir).await {
                                            error!("creating data dir failed with error, {e}");
                                        }
                                    }

                                    data_dir.push(format!("{}.json", project_name));
                                    match File::create(&data_dir).await {
                                        Ok(mut file) => match file.write_all(json.as_bytes()).await
                                        {
                                            Ok(_) => info!(
                                                "saved '{project_name}' to file {}.json.",
                                                project_name
                                            ),
                                            Err(e) => {
                                                error!(
                                                    "writing seqeunce data from project to file, {}, failed with an error, {e}",
                                                    data_dir
                                                        .file_name()
                                                        .map(|name| name
                                                            .to_string_lossy()
                                                            .to_string())
                                                        .unwrap_or(format!(
                                                            "{}.json",
                                                            project_name
                                                        ))
                                                )
                                            }
                                        },
                                        Err(e) => {
                                            error!(
                                                "creating file to save sequence to failed with an error, {e}"
                                            )
                                        }
                                    }
                                } else {
                                    error!(
                                        "the '$HOME' env var could not be found. so no xdg dir could be set"
                                    );
                                }
                            }
                            Err(e) => {
                                error!("attept to json-ify, failed with error, {e}");
                            }
                        }
                    }
                    SequencerControlCmd::ListSavedProjects { responder } => {
                        let xdg_dirs = BaseDirectories::new();

                        if let Some(mut data_dir) = xdg_dirs.data_home {
                            data_dir.push("midi-daw");
                            data_dir.push("projects");

                            if let Ok(mut dir_contents) = read_dir(data_dir).await {
                                let mut contents = Vec::default();

                                while let Some(f_name) = dir_contents.next().await {
                                    if let Ok(fname) = f_name {
                                        if let Ok(ftype) = fname.file_type().await
                                            && ftype.is_file()
                                        {
                                            contents.push(
                                                fname.file_name().to_string_lossy().to_string(),
                                            )
                                        }
                                    }
                                }

                                if let Err(e) = responder.send(contents.clone()) {
                                    error!(
                                        "attempts to respond to front end failed with error: {e:?}"
                                    );
                                } else {
                                    info!("listed projects {:?}", contents);
                                }
                            } else {
                                error!("failed to read saved files in data directory.");
                            }
                        } else {
                            error!(
                                "the '$HOME' env var could not be found. so no xdg dir could be set"
                            );
                        }
                    }
                    SequencerControlCmd::LoadSavedProject { project_name } => {
                        // get file path
                        let xdg_dirs = BaseDirectories::new();

                        if let Some(mut data_dir) = xdg_dirs.data_home {
                            data_dir.push("midi-daw");
                            data_dir.push("projects");
                            data_dir.push(format!("{}.json", project_name));

                            // read file from disk
                            if let Ok(file) = File::open(&data_dir).await {
                                let mut reader = BufReader::new(file);

                                let mut json_text = String::new();
                                // let mut last_f_len = json_text.len();
                                if reader.read_to_string(&mut json_text).await.is_ok() {
                                    // let mut f_len = json_text.len();
                                    //
                                    // while last_f_len != f_len {
                                    //     last_f_len = f_len;
                                    //     reader.read_line(&mut json_text);
                                    //     f_len = json_text.len();
                                    // }

                                    // parse JSON
                                    if let Ok(json) =
                                        serde_json::from_str::<AllSequences>(&json_text)
                                    {
                                        sequences.extend(json.clone().into_iter());
                                        info!("restored project, '{}', from disk", project_name);
                                    } else {
                                        warn!("{json_text}");
                                        error!("parsing stored json failed.");
                                    }
                                } else {
                                    error!("failed to read file");
                                }
                            } else {
                                error!(
                                    "failed to open file. does, '{}', exist?",
                                    data_dir.to_string_lossy()
                                );
                            }
                        } else {
                            error!(
                                "the '$HOME' env var could not be found. so no xdg dir could be set"
                            );
                        }
                    }
                    SequencerControlCmd::RmSavedProject { project_name } => {
                        // get file path
                        let xdg_dirs = BaseDirectories::new();

                        if let Some(mut data_dir) = xdg_dirs.data_home {
                            data_dir.push("midi-daw");
                            data_dir.push("projects");
                            // data_dir.push(format!("{}.json", project_name));

                            if !project_name.ends_with(".json") {
                                data_dir.push(format!("{}.json", project_name));
                            } else {
                                data_dir.push(project_name);
                            }

                            if data_dir.exists() {
                                if let Err(e) = remove_file(&data_dir).await {
                                    error!(
                                        "removing, '{}', failed with error, {e}",
                                        data_dir.to_string_lossy()
                                    );
                                } else {
                                    info!("seqeunce file removed");
                                }
                            } else {
                                warn!(
                                    "the project file path: '{}' doesn't exist, can't remove",
                                    data_dir.to_string_lossy()
                                );
                            }
                        } else {
                            error!(
                                "the '$HOME' env var could not be found. so no xdg dir could be set"
                            );
                        }
                    }
                }
            }
        }
    }
}

async fn save_seqeunce(seq: &Sequence, sequence_name: &str) {
    match serde_json::to_string(seq) {
        Ok(json) => {
            let xdg_dirs = BaseDirectories::new();

            if let Some(mut data_dir) = xdg_dirs.data_home {
                data_dir.push("midi-daw");

                if !data_dir.exists() {
                    if let Err(e) = create_dir_all(&data_dir).await {
                        error!("creating data dir failed with error, {e}");
                    }
                }

                data_dir.push(format!("{}.json", seq.name));

                match File::create(&data_dir).await {
                    Ok(mut file) => match file.write_all(json.as_bytes()).await {
                        Ok(_) => info!("saved '{sequence_name}' to file {}.", seq.name),
                        Err(e) => {
                            error!(
                                "writing seqeunce data from seqeunce, {}, to file, {}, failed with an error, {e}",
                                seq.name,
                                data_dir
                                    .file_name()
                                    .map(|name| name.to_string_lossy().to_string())
                                    .unwrap_or(format!("{}.json", seq.name))
                            )
                        }
                    },
                    Err(e) => {
                        error!("creating file to save sequence to failed with an error, {e}")
                    }
                }
            } else {
                error!("the '$HOME' env var could not be found. so no xdg dir could be set");
            }
        }
        Err(e) => {
            error!("attept to json-ify, failed with error, {e}");
        }
    }
}
