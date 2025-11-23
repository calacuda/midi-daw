use crossbeam::channel::{Receiver, Sender, unbounded};
use midi_daw_types::{MidiReqBody, NoteDuration};
use std::{
    sync::{Arc, RwLock},
    thread::{sleep, spawn},
    time::Duration,
};
use tracing::*;
use tungstenite::connect;

use crate::tracks::Track;

pub const BASE_URL: &str = "localhost:8080";

#[derive(Debug)]
pub enum MessageToPlayer {
    PlaySection(usize),
    PlayPattern(usize),
    StopSection(usize),
    StopPattern(usize),
    GetDevs(Sender<Vec<String>>),
}

pub fn playback(sections: Arc<RwLock<Vec<Track>>>, recv: Receiver<MessageToPlayer>) {
    let mut playing: Vec<(usize, usize)> = Vec::default();
    let (send, sync_pulse) = unbounded();
    let _sync_pulse_jh = spawn(move || {
        loop {
            sync_puse_reader(send.clone());
            sleep(Duration::from_secs(1));
        }
    });
    let mut note_threads = Vec::new();
    let client = reqwest::blocking::Client::new();
    info!("playback function setup complete");

    loop {
        // info!("loop");
        if let Ok(message) = recv.try_recv() {
            match message {
                MessageToPlayer::PlaySection(section) => {
                    info!("playing section: {section}");
                    playing.push((section, 0))
                }
                MessageToPlayer::StopSection(section) => {
                    info!("will no longer play section: {section}");
                    playing.retain(|elm| elm.0 != section)
                }
                MessageToPlayer::GetDevs(responder) => {
                    info!("get devs");
                    let midi_url = format!("http://{BASE_URL}/midi");
                    info!("{midi_url}");

                    let devs = if let Ok(req) = client.get(midi_url).send() {
                        info!("devs req {:?}", req);
                        // Vec::new()

                        if let Ok(dev_list) = req.json::<Vec<String>>() {
                            info!("devs {:?}", dev_list);
                            // let mut devs = known_midi_devs.write();
                            // *devs = dev_list;
                            dev_list
                        } else {
                            error!(
                                "returned device list was expected to be json but failed to parse as such."
                            );
                            Vec::new()
                        }
                    } else {
                        error!("failed to make get request to get a list of devices.");
                        Vec::new()
                    };

                    if let Err(e) = responder.send(devs) {
                        error!("{e}");
                    }
                }
                _ => error!("pattern playback not yet implemented"),
            }
        }

        // if rest done
        if let Ok(_) = sync_pulse.try_recv() {
            // for playing, play next
            if let Ok(sections) = sections.read() {
                for (section_i, step_i) in playing.iter_mut() {
                    let section = &sections[*section_i];
                    let steps = &section.steps;
                    let step = &steps[*step_i];
                    let (dev, channel) =
                        (sections[*section_i].dev.clone(), sections[*section_i].chan);
                    let (notes, vel) = (step.note.clone(), step.velocity.unwrap_or(85));
                    let duration = if !section.is_drum {
                        NoteDuration::Sn(1)
                    } else {
                        NoteDuration::Sn(1)
                    };

                    // if notes.len() > 0 {
                    // info!("{notes:?}");
                    // }

                    for note in notes {
                        let req_body = MidiReqBody::new(
                            dev.clone(),
                            channel,
                            midi_daw_types::MidiMsg::PlayNote {
                                note,
                                velocity: vel,
                                duration,
                            },
                        );

                        // mk request
                        let req = client
                            .post(format!("http://{BASE_URL}/midi"))
                            .json(&req_body);

                        note_threads.push(spawn(|| req.send()));
                    }

                    *step_i += 1;
                    *step_i %= steps.len();

                    note_threads.retain(|thread| !thread.is_finished());
                }
            }
        }
    }
}

pub fn sync_puse_reader(tx: Sender<()>) -> () {
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

    // let mut pulses = 0;
    let beats: Vec<String> = vec![
        "1", "1e", "1&", "1a", "2", "2e", "2&", "2a", "3", "3e", "3&", "3a", "4", "4e", "4&", "4a",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    loop {
        let Ok(msg) = socket.read() else {
            return;
        };

        // if pulses % bpq == 0 {
        //     // info!("pulse");
        //     _ = tx.send(());
        // }

        match msg {
            // tungstenite::Message::Binary(_) => pulses += 1,
            tungstenite::Message::Text(msg) => {
                if beats.contains(&msg.to_string()) {
                    _ = tx.send(());
                }
            }
            _ => {}
        }
    }
}
