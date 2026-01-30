use crate::{
    midi::{dev::new_midi_dev, out::midi_out},
    sequencer::sequencer_start,
};
use crossbeam::channel::unbounded;
use std::{
    sync::{Arc, RwLock},
    thread::spawn,
};

pub mod midi;
pub mod sequencer;
pub mod server;

// const APP_NAME: &str = "MIDI-DAW";

#[actix::main]
pub async fn run() -> std::io::Result<()> {
    // tempo
    let tempo = Arc::new(RwLock::new(99.0));
    let bpq = Arc::new(RwLock::new(24.0));
    let pulse_counter = Arc::new(RwLock::new(0));

    // prepare mpsc.
    let (midi_msg_out_tx, midi_msg_out_rx) = unbounded();
    let (new_midi_dev_tx, new_midi_dev_rx) = unbounded();
    let (sequencer_tx, sequencer_rx) = unbounded();

    let (_jh_1, _jh_2, _jh_3) = {
        // start midi output thread.
        let midi_out_jh = spawn({
            let tempo = tempo.clone();
            let bpq = bpq.clone();

            move || midi_out(midi_msg_out_rx, new_midi_dev_rx, tempo, bpq, pulse_counter)
        });

        // start a thread for midi device discovery.
        let new_midi_dev_tx = new_midi_dev_tx.clone();
        let midi_dev_jh = spawn(move || new_midi_dev(new_midi_dev_tx));

        // start a automation thread.
        // let (automation_tx, automation_rx) = unbounded();
        //
        // let automation_jh = spawn(move || automation(automation_rx, midi_msg_out_tx.clone()));

        // start sequencer
        let sequencer_jh = spawn({
            let tempo = tempo.clone();
            let bpq = bpq.clone();

            move || sequencer_start(tempo, bpq, sequencer_rx)
        });

        (midi_out_jh, midi_dev_jh, sequencer_jh)
    };

    // run webserver.
    server::run(tempo, bpq, midi_msg_out_tx, new_midi_dev_tx, sequencer_tx).await
}
