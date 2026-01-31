use std::time::Duration;

use actix::dev::OneshotSender;
use crossbeam::channel::Receiver;
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

use crate::midi::out::unwrap_rw_lock;

pub type AllSequences = FxHashMap<SequenceName, Sequence>;

// #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SequencerControlCmd {
    GetSequence {
        seqeunce: SequenceName,
        responder: OneshotSender<Option<Sequence>>,
    },
    GetSequences {
        responder: OneshotSender<Vec<String>>,
    },
    NewSeqeunce {
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
}

#[tokio::main]
pub async fn sequencer_start(tempo: Tempo, bpq: BPQ, controls: Receiver<SequencerControlCmd>) {
    // let url = Uri::new("/tmp/hyperlocal.sock", "/").into();

    let mk_timer = || {
        // calculate sleep_time based on BPQ & tempo.
        let (tempo, beats) = (unwrap_rw_lock(&tempo, 99.), unwrap_rw_lock(&bpq, 24.));
        let sleep_time = Duration::from_secs_f64((60.0 / tempo) / 4. / beats);

        move || {
            std::thread::sleep(sleep_time);
        }
    };

    // start a sleep thread to sleep for sleep_time.
    let mut sleep_thread = std::thread::spawn(mk_timer());
    let mut counter = 0.;

    let mut sequences: AllSequences = FxHashMap::default();
    let mut queued_sequences: Vec<SequenceName> = Vec::default();
    let mut playing_sequences: Vec<SequenceName> = Vec::default();
    let mut jh_s = Vec::default();

    loop {
        if sleep_thread.is_finished() {
            sleep_thread = std::thread::spawn(mk_timer());

            playing_sequences.append(&mut queued_sequences);

            if (counter % (unwrap_rw_lock(&bpq, 24.) / 4.)) == 0.0 {
                let i = counter / (unwrap_rw_lock(&bpq, 24.) / 4.);
                // info!("i = {i}");
                // info!("i % 16 = {}", i as usize % 16);

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
            }

            counter += 1.;
            counter %= f64::MAX;
        } else {
            while let Ok(msg) = controls.try_recv() {
                // do msg thing

                match msg {
                    SequencerControlCmd::GetSequence {
                        seqeunce,
                        responder,
                    } => {
                        if let Err(e) =
                            responder.send(sequences.get(&seqeunce).map(|seq| seq.clone()))
                        {
                            error!("sending seqeunce failed with error: {e:?}");
                        }
                    }
                    SequencerControlCmd::GetSequences { responder } => {
                        if let Err(e) =
                            responder.send(sequences.keys().map(|name| name.clone()).collect())
                        {
                            error!("sending seqeunce names failed with error: {e:?}");
                        }
                    }
                    SequencerControlCmd::NewSeqeunce {
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
                    }
                    SequencerControlCmd::Play(names) => names.into_iter().for_each(|name| {
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
                    }
                    SequencerControlCmd::StopAll => {
                        queued_sequences.clear();
                        playing_sequences.clear();
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

                                    *msg_note == note
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
                }
            }
        }
    }
}
