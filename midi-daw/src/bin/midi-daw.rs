use bevy::prelude::*;
use crossbeam::channel::unbounded;
use midi_daw::midi::{dev::new_midi_dev, out::midi_out};
use midi_daw_lib::{MidiOutput, NewMidiDev};
use std::thread::spawn;

fn main() {
    // TODO: start thread to handle midi output and dev creation.
    // also store the crossbeam rx & tx channel in a Resource

    // prepare mpsc.
    let (midi_msg_out_tx, midi_msg_out_rx) = unbounded();
    let (new_midi_dev_tx, new_midi_dev_rx) = unbounded();

    let (_jh_1, _jh_2 /* _jh_3 */) = {
        // start midi output thread.
        let midi_out_jh = spawn(move || midi_out(midi_msg_out_rx, new_midi_dev_rx));

        // start a thread for midi device discovery.
        let new_midi_dev_tx = new_midi_dev_tx.clone();
        let midi_dev_jh = spawn(move || new_midi_dev(new_midi_dev_tx));

        // // start a automation thread.
        // let (automation_tx, automation_rx) = unbounded();
        //
        // let automation_jh = spawn(move || automation(automation_rx, midi_msg_out_tx.clone()));

        (midi_out_jh, midi_dev_jh)
    };

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(NewMidiDev(new_midi_dev_tx))
        .insert_resource(MidiOutput(midi_msg_out_tx))
        .add_systems(Startup, setup)
        .add_systems(Update, ())
        .run();
}
