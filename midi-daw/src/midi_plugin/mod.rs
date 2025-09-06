use crate::{
    CmdPallet, EdittingCell, FirstViewTrack, MidiNote, MidiOutput, N_STEPS, Playing, Tempo, Track,
    TrackID, TrackerCmd, playing,
};
use bevy::prelude::*;
use core::time::Duration;
use midi_daw_types::MidiDeviceName;
use midi_msg::{Channel, ChannelVoiceMsg, MidiMsg};
use std::time::Instant;

#[derive(Resource, Clone, Debug, Copy, Eq, PartialEq)]
pub struct SyncPulse {
    last_pulse_time: Instant,
    pub n_pulses: usize,
}

#[derive(Resource, Clone, Debug, Eq, PartialEq)]
pub struct SyncTimer(Timer);

#[derive(Component, Clone, Debug, Copy, Eq, Hash, PartialEq)]
pub struct PlayingTrack(pub usize, pub usize, pub Option<usize>); // track index, step index,

#[derive(Resource, Clone, Debug, Eq, PartialEq)]
pub struct ControllerName(String);

#[derive(Resource, Clone, Debug, Copy, Eq, Hash, PartialEq)]
pub struct LastPlayedPulse(Option<usize>);

#[derive(Resource, Clone, Debug, Copy, Eq, Hash, PartialEq)]
pub struct BPQ(pub usize);

#[derive(Resource, Clone, Debug, Copy, Eq, Hash, PartialEq, Deref, DerefMut)]
pub struct PlayingSyncPulse(pub bool);

// #[derive(Resource, Clone, Debug, Copy, Eq, Hash, PartialEq)]
// pub struct PlayHead

#[derive(Component, Clone, Debug, Copy, Eq, Hash, PartialEq)]
pub struct PlayingQueued;

#[derive(Component, Clone, Debug, Copy, Eq, Hash, PartialEq)]
pub struct QueueStopPlaying;

#[derive(Component, Clone, Debug, Eq, PartialEq)]
struct NoteOff {
    note: MidiNote,
    velocity: u8,
    /// when to send note off command in pulses,
    when: usize,
    device: MidiDeviceName,
    channel: Channel,
}

pub struct MidiOutPlugin;

impl Plugin for MidiOutPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SyncPulse {
            last_pulse_time: Instant::now(),
            n_pulses: 0,
        })
        .insert_resource(LastPlayedPulse(None))
        .insert_resource(Tempo(120))
        .insert_resource(SyncTimer(Timer::new(
            Duration::from_secs_f64(60.0 / 120.0 / 48.0),
            TimerMode::Repeating,
        )))
        .insert_resource(Playing(true))
        .insert_resource(BPQ(48))
        // .insert_resource(LastPlayedPulse(None))
        .insert_resource(PlayingSyncPulse(true))
        .insert_resource(CmdPallet(false))
        .insert_resource(EdittingCell(false))
        .init_resource::<FirstViewTrack>()
        .add_systems(Startup, setup)
        .add_systems(Update, sync.run_if(sync_pulsing))
        .add_systems(
            Update,
            (
                (
                    send_notes.run_if(playing.and(not_played_yet)),
                    // note_notif.run_if(playing),
                    // update_front_end.run_if(sync_pulsing)
                )
                    // .chain()
                    .run_if(on_thirtysecond_note),
                handle_note_off.run_if(notes_are_held),
            ),
        );
    }
}

fn setup(tempo: Res<Tempo>, mut sync_timer: ResMut<SyncTimer>, bpq: Res<BPQ>) {
    sync_timer.0 = Timer::new(
        Duration::from_secs_f64(60.0 / tempo.0 as f64 / bpq.0 as f64),
        TimerMode::Repeating,
    );

    // set playback cursor loc.
}

fn sync_pulsing(pulsing: Res<PlayingSyncPulse>) -> bool {
    **pulsing
}

fn notes_are_held(held_notes: Query<&NoteOff>) -> bool {
    held_notes.iter().len() != 0
}

fn sync(
    mut sync_timer: ResMut<SyncTimer>,
    time: Res<Time>,
    tempo: Res<Tempo>,
    mut pulse: ResMut<SyncPulse>,
    bpq: Res<BPQ>,
) {
    sync_timer.0.tick(time.delta());

    if sync_timer.0.just_finished() {
        pulse.n_pulses += 1;
        pulse.n_pulses %= usize::MAX;

        sync_timer.0.set_duration(Duration::from_secs_f64(
            60.0 / tempo.0 as f64 / bpq.0 as f64,
        ));
    }
}

pub fn on_thirtysecond_note(pulse: Res<SyncPulse>, bpq: Res<BPQ>) -> bool {
    // info!("n_pulses {}", pulse.n_pulses);
    // 6 because 24 beats is a quarter note.
    pulse.n_pulses % (bpq.0 / 4) == 0
}

fn not_played_yet(last_played: Res<LastPlayedPulse>, pulse: Res<SyncPulse>) -> bool {
    // info!(
    //     "n_pulses: {}, last_played: {}",
    //     pulse.n_pulses, last_played.0
    // );

    if let Some(lp) = last_played.0 {
        debug!("n_pulses: {}, last_played: {}", pulse.n_pulses, lp);

        pulse.n_pulses > lp
    } else {
        true
    }
}

// fn update_front_end(mut state_updated: EventWriter<StateUpdated>) {
//     state_updated.write_default();
// }

// fn note_notif() {
//     info!("note");
// }

fn handle_note_off(
    mut commands: Commands,
    held_notes: Query<(Entity, &NoteOff)>,
    pulse: Res<SyncPulse>,
    mut midi_out: ResMut<MidiOutput>,
) {
    held_notes.into_iter().for_each(|(entity, note_off)| {
        if pulse.n_pulses == note_off.when {
            midi_out.send(
                note_off.device.clone(),
                MidiMsg::ChannelVoice {
                    channel: note_off.channel,
                    msg: ChannelVoiceMsg::NoteOff {
                        note: note_off.note,
                        velocity: note_off.velocity,
                    },
                },
            );
            commands.entity(entity).despawn();
        }
    });
}

fn send_notes(
    mut commands: Commands,
    tracks: Query<(&Track, &TrackID)>,
    pulse: Res<SyncPulse>,
    bpq: Res<BPQ>,
    mut midi_out: ResMut<MidiOutput>,
) {
    let step_i = get_step_num(&pulse, &bpq);
    // let last_step_i = if step_i > 0 { step_i - 1 } else { N_STEPS - 1 };

    for (ref track, id) in tracks.iter() {
        if id.playing {
            if let Some((Some(note), cmds)) = track
                .steps
                .get(step_i)
                .map(|step| (step.note, step.cmds.clone()))
            {
                let velocity = 111;
                let sixteenth_note = bpq.0 / 4;
                let mut when_off = if usize::MAX - sixteenth_note >= pulse.n_pulses {
                    pulse.n_pulses + sixteenth_note
                } else {
                    sixteenth_note - (usize::MAX - pulse.n_pulses)
                } % usize::MAX;

                // for cmd in [cmds.0, cmds.1] {
                // }

                let hold_steps = match cmds {
                    (TrackerCmd::HoldFor { notes: n1 }, TrackerCmd::HoldFor { notes: n2 }) => {
                        Some(n1.max(n2))
                    }
                    (TrackerCmd::HoldFor { notes }, _) => Some(notes),
                    (_, TrackerCmd::HoldFor { notes }) => Some(notes),
                    _ => None,
                };

                if let Some(hold_for) = hold_steps {
                    let amt = if hold_for.0 > 0 {
                        sixteenth_note * (hold_for.0 + 1)
                    } else {
                        sixteenth_note + sixteenth_note / 2
                    };

                    if usize::MAX - amt >= pulse.n_pulses {
                        when_off = amt + pulse.n_pulses;
                    } else {
                        when_off = amt - (usize::MAX - pulse.n_pulses);
                    }

                    // when_off %= usize::MAX;
                }

                midi_out.send(
                    track.dev.clone(),
                    MidiMsg::ChannelVoice {
                        channel: track.chan,
                        msg: ChannelVoiceMsg::NoteOn { note, velocity },
                    },
                );
                commands.spawn(NoteOff {
                    note,
                    velocity,
                    device: track.dev.clone(),
                    channel: track.chan,
                    when: when_off,
                });

                // log.write(Log::error(format!("playing: {note}")));
            }
        }
    }
}

pub fn get_step_num(pulse: &Res<SyncPulse>, bpq: &Res<BPQ>) -> usize {
    (pulse.n_pulses / (bpq.0 / 4)) % N_STEPS
}

// fn toggle_playing(
//     mut cmds: Commands,
//     // buttons: Single<&Gamepad>,
//     // mut playing_state: ResMut<NextState<PlayingState>>,
//     // current_play_state: Res<State<PlayingState>>,
//     // mut playing_sredraw_displayync: ResMut<PlayingSyncPulse>,
//     playing: Query<(Entity, &PlayingPhrase)>,
//     // screen: Res<Screen>,
// ) {
//     let start_button = GamepadButton::Start;
//
//     if buttons.just_released(start_button) && !buttons.pressed(GamepadButton::Mode) {
//         match *screen {
//             Screen::EditPhrase(phrase_n) => {
//                 if *current_play_state.get() != PlayingState::Playing {
//                     playing_state.set(PlayingState::Playing);
//                 }
//                 // else if *current_play_state.get() != PlayingState::Playing {
//                 //     playing_state.set(PlayingState::NotPlaying);
//                 // }
//
//                 // playing_sync.0 = true;
//                 // info!("starting sync pulse");
//
//                 let maybe_playing = playing
//                     .iter()
//                     .find(|(_entity, playing)| playing.0 == phrase_n);
//
//                 if let Some((already_playing, _)) = maybe_playing {
//                     info!("stop playback event queued for: {phrase_n}");
//                     cmds.entity(already_playing).insert(QueueStopPlaying);
//                 } else {
//                     info!("queuing playing for: {phrase_n}");
//                     cmds.spawn((PlayingPhrase(phrase_n, 0, None), PlayingQueued));
//                 }
//             }
//             _ => {}
//         };
//     }
// }

fn should_play_queue(pulse: Res<SyncPulse>, bpq: Res<BPQ>) -> bool {
    let to_play = ((pulse.n_pulses / (bpq.0 / 4)) % 16) == 0;

    // info!("to_play = {to_play}, {}", pulse.n_pulses);

    to_play
}

// fn play_queued(
//     mut cmds: Commands,
//     playing_queue: Query<(Entity, &PlayingPhrase), With<PlayingQueued>>,
// ) {
//     for (id, phrase) in playing_queue {
//         info!("playing queued phrase: {}", phrase.0);
//
//         cmds.entity(id).remove::<PlayingQueued>();
//     }
// }

// fn stop_queued(
//     mut cmds: Commands,
//     stop_queue: Query<(Entity, &PlayingPhrase), With<QueueStopPlaying>>,
// ) {
//     for (id, phrase) in stop_queue {
//         info!("stopping queued phrase: {}", phrase.0);
//
//         cmds.entity(id).despawn();
//     }
// }

// fn stop_playing(
//     // buttons: Single<&Gamepad>,
//     mut playing_state: ResMut<NextState<PlayingState>>,
//     playing: Query<&PlayingPhrase>,
// ) {
//     // let start_button = GamepadButton::Start;
//     //
//     // if buttons.just_released(start_button) {
//     //     playing_state.set(PlayingState::NotPlaying);
//     // }
//
//     if playing.iter().len() == 0 {
//         info!("stopping playback");
//         playing_state.set(PlayingState::NotPlaying);
//     }
// }

// fn toggle_syncing(buttons: Single<&Gamepad>, mut playing_sync: ResMut<PlayingSyncPulse>) {
//     let select_button = GamepadButton::Select;
//
//     if buttons.just_released(select_button) && !buttons.pressed(GamepadButton::Mode) {
//         playing_sync.0 = !playing_sync.0;
//         info!("playing sync: {}", playing_sync.0);
//     }
// }
