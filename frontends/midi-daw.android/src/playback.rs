use crossbeam::channel::{Receiver, Sender, unbounded};
use midi_daw_types::MidiReqBody;
use std::{
    sync::{Arc, RwLock},
    thread::{sleep, spawn},
    time::Duration,
};
use tracing::*;
use tungstenite::connect;

use crate::tracks::Track;

pub const BASE_URL: &str = "localhost:8080";

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum MessageToPlayer {
    PlaySection(usize),
    PlayPattern(usize),
    StopSection(usize),
    StopPattern(usize),
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

    loop {
        if let Ok(message) = recv.recv() {
            match message {
                MessageToPlayer::PlaySection(section) => playing.push((section, 0)),
                MessageToPlayer::StopSection(section) => playing.retain(|elm| elm.0 != section),
                _ => error!("pattern playback not yet implemented"),
            }
        }

        // if rest done
        if let Ok(_) = sync_pulse.recv() {
            // for playing, play next
            if let Ok(sections) = sections.read() {
                let client = reqwest::blocking::Client::new();

                for (section, step_i) in playing.iter_mut() {
                    let step = &sections[*section].steps[*step_i];
                    let (dev, channel) = (sections[*section].dev.clone(), sections[*section].chan);
                    let (notes, vel) = (step.note.clone(), step.velocity.unwrap_or(85));

                    for note in notes {
                        let req_body = MidiReqBody::new(
                            dev.clone(),
                            channel,
                            midi_daw_types::MidiMsg::PlayNote {
                                note,
                                velocity: vel,
                                duration: midi_daw_types::NoteDuration::Sn(1),
                            },
                        );

                        if let Ok(body) = serde_json::to_string(&req_body) {
                            // mk request
                            let req = client.post(format!("http://{BASE_URL}/midi")).body(body);

                            note_threads.push(spawn(|| req.send()));
                        }
                    }

                    *step_i += 1;

                    note_threads.retain(|thread| !thread.is_finished());
                }
            }
        }
    }
}

pub fn sync_puse_reader(tx: Sender<()>) -> () {
    let bpq = 24;
    let (mut socket, response) = match connect(format!("ws://{BASE_URL}/message-bus")) {
        Ok(val) => val,
        Err(e) => {
            error!("{e}");
            return;
        }
    };

    if response.status() != 200 {
        return;
    }

    info!("connected...");

    let mut pulses = 0;

    loop {
        let Ok(msg) = socket.read() else {
            return;
        };

        if pulses % bpq == 0 {
            info!("pulse");
            _ = tx.send(());
        }

        match msg {
            tungstenite::Message::Binary(_) => pulses += 1,
            _ => {}
        }
    }
}
