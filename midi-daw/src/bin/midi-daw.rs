use bevy::prelude::*;
use crossbeam::channel::unbounded;
use midi_daw::midi::{MidiDev, dev::new_midi_dev, out::midi_out};
use midi_daw_lib::{
    CursorLocation, DisplayStart, MidiCmd, MidiOutput, N_STEPS, NewMidiDev, Step, Track, TrackID,
    midi_plugin::MidiOutPlugin,
};
use midi_msg::Channel;
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

    new_midi_dev_tx
        .send(MidiDev::CreateVirtual("TEST-DEV".into()))
        .unwrap();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(MidiOutPlugin)
        .insert_resource(NewMidiDev(new_midi_dev_tx))
        .insert_resource(MidiOutput(midi_msg_out_tx))
        .init_resource::<CursorLocation>()
        .init_resource::<DisplayStart>()
        .add_systems(Startup, (setup_tracks, setup_track_dis, setup_cursor))
        // .add_systems(Update, ())
        .run();
}

fn setup_tracks(mut cmds: Commands) {
    let mut steps: Vec<Step<MidiCmd>> = (1..N_STEPS).map(|_| Step::default()).collect();

    let note = [48, 52, 55, 59];
    for (i, step) in steps.iter_mut().step_by(8).enumerate() {
        step.note = Some(note[i % 4]);
    }

    let track = Track::Midi {
        steps,
        dev: "TEST-DEV".into(),
        chan: Channel::Ch1,
    };

    cmds.spawn((
        TrackID {
            id: 0,
            playing: true,
        },
        track,
        // Track::default(),
    ));
    cmds.spawn((
        TrackID {
            id: 1,
            playing: true,
        },
        Track::default(),
    ));
    // cmds.spawn((TrackID(2), Track::default()));
    // cmds.spawn((TrackID(3), Track::default()));
}

fn setup_cursor(mut cmds: Commands) {
    // cmds.spawn((
    //     TextComponent {
    //         text: ">".into(),
    //         point: Point::new(x_from_col(2), row_from_line(2)),
    //         color: Some(Rgb565::CYAN),
    //         ..default()
    //     },
    //     CursorText,
    // ));
}

fn setup_track_dis(mut cmds: Commands) {}
