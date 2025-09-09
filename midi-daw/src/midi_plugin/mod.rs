use crate::{
    CmdPallet, EdittingCell, FirstViewTrack, MidiNote, MidiOutput, N_STEPS, Playing, StopHeldNotes,
    Tempo, Track, TrackID, TrackerCmd, playing,
};
use bevy::prelude::*;
use core::time::Duration;
use midi_daw_types::MidiDeviceName;
use midi_msg::{Channel, ChannelVoiceMsg, ControlChange, MidiMsg};
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

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct RollNote;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct RepeatNote;

#[derive(Component, Deref, DerefMut, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct RCounter(pub usize);

#[derive(Component, Clone, Debug, Eq, PartialEq)]
pub struct RNote {
    note: MidiNote,
    velocity: u8,
    /// when to send note off command in pulses,
    when: usize,
    device: MidiDeviceName,
    channel: Channel,
}

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
        app.add_event::<StopHeldNotes>()
            .insert_resource(SyncPulse {
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
                        // send_repeat_notes.run_if(playing.and(not_played_yet)),
                        send_r_notes::<RepeatNote>().run_if(playing.and(not_played_yet)),
                        // note_notif.run_if(playing),
                        // update_front_end.run_if(sync_pulsing)
                    )
                        // .chain()
                        .run_if(on_step),
                    send_r_notes::<RollNote>()
                        .run_if(playing.and(not_played_yet).and(on_half_step)),
                    handle_all_notes_off.run_if(on_event::<StopHeldNotes>),
                    handle_note_off.run_if(notes_are_held.and(on_step.or(on_half_step))),
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

pub fn on_step(pulse: Res<SyncPulse>, bpq: Res<BPQ>) -> bool {
    // on_thirtysecond_note(pulse, bpq)
    on_sixteenth_note(&pulse, &bpq)
}

pub fn on_half_step(pulse: Res<SyncPulse>, bpq: Res<BPQ>) -> bool {
    on_sixteenth_note(&pulse, &bpq) || on_thirtysecond_note(&pulse, &bpq)
}

pub fn on_sixteenth_note(pulse: &SyncPulse, bpq: &BPQ) -> bool {
    // info!("n_pulses {}", pulse.n);
    // 6 because 24 beats is a quarter note.
    pulse.n_pulses % (bpq.0 / 4) == 0
}

pub fn on_thirtysecond_note(pulse: &SyncPulse, bpq: &BPQ) -> bool {
    // info!("n_pulses {}", pulse.n_pulses);
    // 6 because 24 beats is a quarter note.
    pulse.n_pulses % (bpq.0 / 8) == 0
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

fn send_r_notes<T>()
-> impl Fn(Commands, Query<(Entity, &RNote, &mut RCounter), With<T>>, ResMut<MidiOutput>, Res<SyncPulse>)
where
    T: Component,
{
    fn do_send_r_notes<T>(
        mut commands: Commands,
        notes: Query<(Entity, &RNote, &mut RCounter), With<T>>,
        mut midi_out: ResMut<MidiOutput>,
        pulse: Res<SyncPulse>,
    ) where
        T: Component,
    {
        for (entity, note_conf, mut counter) in notes {
            if counter.0 > 0 {
                let RNote {
                    note,
                    velocity,
                    when,
                    device,
                    channel,
                } = note_conf.clone();

                midi_out.send(
                    device.clone(),
                    MidiMsg::ChannelVoice {
                        channel,
                        msg: ChannelVoiceMsg::NoteOn { note, velocity },
                    },
                );

                commands.spawn(NoteOff {
                    note,
                    velocity,
                    device,
                    channel,
                    when: when + pulse.n_pulses,
                });

                counter.0 -= 1;
            } else {
                commands.entity(entity).despawn();
            }
        }
    }

    do_send_r_notes
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

fn handle_all_notes_off(
    mut commands: Commands,
    held_notes: Query<(Entity, &NoteOff)>,
    r_notes: Query<(Entity, &RNote), With<RCounter>>,
    // pulse: Res<SyncPulse>,
    mut midi_out: ResMut<MidiOutput>,
    mut evs: EventReader<StopHeldNotes>,
) {
    held_notes.into_iter().for_each(|(entity, note_off)| {
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
    });

    // info!("stopping all");
    r_notes.into_iter().for_each(|(entity, r_note)| {
        midi_out.send(
            r_note.device.clone(),
            MidiMsg::ChannelVoice {
                channel: r_note.channel,
                msg: ChannelVoiceMsg::NoteOff {
                    note: r_note.note,
                    velocity: r_note.velocity,
                },
            },
        );
        commands.entity(entity).despawn();
    });

    evs.clear();
}

fn send_notes(
    mut commands: Commands,
    tracks: Query<(&Track, &TrackID)>,
    pulse: Res<SyncPulse>,
    bpq: Res<BPQ>,
    mut midi_out: ResMut<MidiOutput>,
    mut evs: EventWriter<StopHeldNotes>,
) {
    let step_i = get_step_num(&pulse, &bpq);
    // let last_step_i = if step_i > 0 { step_i - 1 } else { N_STEPS - 1 };

    for (ref track, id) in tracks.iter() {
        if id.playing {
            if let Some(step) = track.steps.get(step_i) {
                let cmds = step.cmds.clone();

                if let Some(note) = step.note {
                    let velocity = 111;
                    let sixteenth_note = bpq.0 / 4;

                    // handle note hold cmd
                    let mut when_off = if usize::MAX - sixteenth_note >= pulse.n_pulses {
                        pulse.n_pulses + sixteenth_note
                    } else {
                        sixteenth_note - (usize::MAX - pulse.n_pulses)
                    } % usize::MAX;

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

                        when_off %= usize::MAX;
                    }

                    // handle chord cmd
                    let mut notes = vec![note];

                    let mut add_notes = |chord_notes: Vec<i8>| {
                        notes.append(
                            &mut chord_notes
                                .clone()
                                .into_iter()
                                .map(|note_offset| {
                                    let new_note = if (note_offset < 0)
                                        && ((note_offset as u8) <= note)
                                    {
                                        note - (note_offset as u8)
                                    } else if (note_offset > 0)
                                        && (127 - (note_offset as u8)) >= note
                                    {
                                        note + (note_offset as u8)
                                    } else if (note_offset < 0) && ((note_offset as u8) > note) {
                                        127 - ((note_offset as u8) - note)
                                    } else {
                                        note + (note_offset as u8)
                                    };

                                    new_note % 128
                                })
                                .collect(),
                        );
                    };

                    match cmds.clone() {
                        (TrackerCmd::Chord { chord: c1 }, TrackerCmd::Chord { chord: c2 }) => {
                            add_notes(c1);
                            add_notes(c2);
                        }
                        (TrackerCmd::Chord { chord }, _) => add_notes(chord),
                        (_, TrackerCmd::Chord { chord }) => add_notes(chord),
                        _ => {}
                    };

                    // handle roll/repeat
                    match {
                        match cmds.clone() {
                            (TrackerCmd::Roll { times: t1 }, TrackerCmd::Roll { times: t2 }) => {
                                Some((TrackerCmd::Roll { times: t1.max(t2) }, false))
                            }
                            (
                                TrackerCmd::Repeat { times: t1 },
                                TrackerCmd::Repeat { times: t2 },
                            ) => Some((TrackerCmd::Repeat { times: t1.max(t2) }, false)),
                            (TrackerCmd::Roll { times }, TrackerCmd::Repeat { times: _ })
                            | (TrackerCmd::Repeat { times: _ }, TrackerCmd::Roll { times }) => {
                                Some((TrackerCmd::Roll { times }, false))
                            }

                            (TrackerCmd::Roll { times }, TrackerCmd::HoldFor { notes: _ }) => {
                                Some((TrackerCmd::Roll { times }, true))
                            }
                            (TrackerCmd::HoldFor { notes: _ }, TrackerCmd::Roll { times }) => {
                                Some((TrackerCmd::Roll { times }, true))
                            }
                            (TrackerCmd::Repeat { times }, TrackerCmd::HoldFor { notes: _ }) => {
                                Some((TrackerCmd::Repeat { times }, true))
                            }
                            (TrackerCmd::HoldFor { notes: _ }, TrackerCmd::Repeat { times }) => {
                                Some((TrackerCmd::Repeat { times }, true))
                            }

                            (TrackerCmd::Roll { times }, _) => {
                                Some((TrackerCmd::Roll { times }, false))
                            }
                            (_, TrackerCmd::Roll { times }) => {
                                Some((TrackerCmd::Roll { times }, false))
                            }
                            (TrackerCmd::Repeat { times }, _) => {
                                Some((TrackerCmd::Repeat { times }, false))
                            }
                            (_, TrackerCmd::Repeat { times }) => {
                                Some((TrackerCmd::Repeat { times }, false))
                            }
                            _ => None,
                        }
                    } {
                        Some((TrackerCmd::Roll { times }, holding)) => {
                            commands.spawn((
                                RNote {
                                    note,
                                    velocity,
                                    when: when_off - pulse.n_pulses,
                                    channel: track.chan,
                                    device: track.dev.clone(),
                                },
                                RCounter(times.0),
                                RollNote,
                            ));

                            if !holding {
                                commands.spawn(NoteOff {
                                    note,
                                    velocity,
                                    device: track.dev.clone(),
                                    channel: track.chan,
                                    when: sixteenth_note / 2,
                                });
                            }
                        }
                        Some((TrackerCmd::Repeat { times }, _)) => {
                            commands.spawn((
                                RNote {
                                    note,
                                    velocity,
                                    when: when_off - pulse.n_pulses,
                                    channel: track.chan,
                                    device: track.dev.clone(),
                                },
                                RCounter(times.0),
                                RepeatNote,
                            ));
                        }
                        _ => {}
                    };

                    for note in notes {
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

                // handle stop cmd
                match cmds {
                    (TrackerCmd::Panic, _) => {
                        // warn!("emmiting StopHeldNotes 1");
                        evs.write_default();
                    }
                    (_, TrackerCmd::Panic) => {
                        // warn!("emmiting StopHeldNotes 2");
                        evs.write_default();
                    }
                    _ => {}
                };

                let mut send_cmd = |cc, arg_1| {
                    midi_out.send(
                        track.dev.clone(),
                        MidiMsg::ChannelVoice {
                            channel: track.chan,
                            msg: ChannelVoiceMsg::ControlChange {
                                control: ControlChange::CC {
                                    control: cc,
                                    value: arg_1,
                                },
                            },
                        },
                    )
                };

                // handle cc cmd
                match cmds {
                    (
                        TrackerCmd::MidiCmd {
                            cc_param: cc_1,
                            arg: a_1,
                        },
                        TrackerCmd::MidiCmd {
                            cc_param: cc_2,
                            arg: a_2,
                        },
                    ) => {
                        send_cmd(cc_1, a_1);
                        send_cmd(cc_2, a_2);
                    }
                    (
                        TrackerCmd::MidiCmd {
                            cc_param: cc_1,
                            arg: a_1,
                        },
                        _,
                    ) => {
                        send_cmd(cc_1, a_1);
                    }
                    (
                        _,
                        TrackerCmd::MidiCmd {
                            cc_param: cc_1,
                            arg: a_1,
                        },
                    ) => {
                        send_cmd(cc_1, a_1);
                    }
                    _ => {}
                };
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chord_logic_corce() {
        for note in [0, 1, 2, 3, 127, 126, 125, 124] {
            let mut notes = Vec::default();

            let mut add_notes = |chord_notes: Vec<i8>| {
                notes.append(
                    &mut chord_notes
                        .clone()
                        .into_iter()
                        .map(|note_offset| {
                            let new_note = if (note_offset < 0) && ((note_offset as u8) <= note) {
                                note - (note_offset as u8)
                            } else if (note_offset > 0) && (127 - (note_offset as u8)) >= note {
                                note + (note_offset as u8)
                            } else if (note_offset < 0) && ((note_offset as u8) > note) {
                                127 - ((note_offset as u8) - note)
                            } else {
                                note + (note_offset as u8)
                            };

                            new_note % 128
                        })
                        .collect(),
                );
            };

            add_notes((i8::MIN..i8::MAX).collect());

            for (i, chord_note) in notes.into_iter().enumerate() {
                assert!(
                    chord_note <= 127,
                    "{chord_note} (ie. greater 127, the max note in midi). this happened when note was {note}, and offset was {}.",
                    // i / n_nums,
                    // i % n_nums,
                    (i8::MIN..i8::MAX).collect::<Vec<i8>>()[i]
                );
            }
        }
    }
}
