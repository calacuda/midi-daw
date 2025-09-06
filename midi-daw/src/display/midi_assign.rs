use crate::{
    CursorLocation, DisplayStart, Screen, ScreenState, Track, TrackID, display::HEADER_TEXT_COLOR,
    north_button_pressed,
};
use bevy::prelude::*;
use midi_daw::midi::dev::fmt_dev_name;
use midir::MidiOutput;

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct OldDevMarker;

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct NewChannelMarker;

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct MenuNodeMarker;

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct NewDevNameMarker(pub usize);

#[derive(Component, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct DevName(pub String);

#[derive(Resource, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct MidiDevMenuCursor(pub usize, pub usize);

pub struct MidiAssignmentPlugin;

impl Plugin for MidiAssignmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                enter_dev_edit.run_if(in_state(ScreenState::MainScreen).and(north_button_pressed)),
                leave_menu.run_if(in_state(ScreenState::EditTrackMidiDev).and(north_button_pressed)),
                display_devs.run_if(in_state(ScreenState::EditTrackMidiDev)),
            )
        )
        .add_systems(
            OnExit(ScreenState::EditTrackMidiDev),
            (
                // set track midi dev
                set_midi_dev,
                tear_down_screen,
                // rm_dev_names,
            ),
        )
        .add_systems(
            OnEnter(ScreenState::EditTrackMidiDev),
            (setup_data, setup_screen, setup_cursor, spawn_dev_names),
        )
        // .add_systems(
        //     Update,
        //     (redraw,).run_if(in_state(ScreenState::EditTrackMidiDev)),
        // )
        ;
    }
}

fn display_devs(
    mut dev_displays: Query<(&mut Text, &NewDevNameMarker)>,
    known_devs: Query<&DevName>,
    cursor: Res<CursorLocation>,
) {
    let mut devs: Vec<String> = known_devs.into_iter().map(|dev| dev.0.clone()).collect();
    devs.sort();
    let off_set = cursor.1;

    for (ref mut text, dev_mark) in dev_displays {
        text.0 = devs[(off_set + dev_mark.0) % devs.len()].clone();
    }
}

fn tear_down_screen(mut commands: Commands, nodes: Query<Entity, With<MenuNodeMarker>>) {
    for entity in nodes {
        commands.entity(entity).despawn();
    }
}

// fn rm_dev_names(mut commands: Commands, devs: Query<Entity, With<DevName>>) {
//     for entity in devs {
//         commands.entity(entity).despawn();
//     }
// }

fn setup_data(
    mut new_screen: ResMut<Screen>,
    display_start: Res<DisplayStart>,
    cursor: Res<CursorLocation>,
) {
    let start = display_start.0;
    let col = cursor.1 / 3;
    let track = start + col;

    *new_screen = Screen::EditTrackMidiDev {
        track,
        dev: None,
        chan: None,
    }
}

pub fn spawn_dev_names(mut commands: Commands, devs: Query<&DevName>) {
    let devs: Vec<String> = devs.into_iter().map(|dev| dev.0.clone()).collect();

    let midi_out = MidiOutput::new("MIDI-DAW-GUI-TRACKER").unwrap();
    midi_out
        .ports()
        .into_iter()
        .filter_map(|port| midi_out.port_name(&port).ok().map(fmt_dev_name))
        .for_each(|name| {
            if !devs.contains(&name) {
                commands.spawn(DevName(name));
            }
        });
}

fn setup_cursor(mut commands: Commands) {
    commands.insert_resource(MidiDevMenuCursor(0, 0));
}

fn enter_dev_edit(mut next_screen: ResMut<NextState<ScreenState>>) {
    next_screen.set(ScreenState::EditTrackMidiDev);
}

fn leave_menu(mut next_screen: ResMut<NextState<ScreenState>>) {
    next_screen.set(ScreenState::MainScreen);
}

fn setup_screen(mut commands: Commands) {
    // A 2D camera for the user interface.
    // This camera will render the UI.
    let ui_camera_entity = commands
        .spawn((
            Camera2d::default(),
            Camera {
                clear_color: ClearColorConfig::Custom(
                    Srgba {
                        red: (30. / 255.),
                        green: (30. / 255.),
                        blue: (46. / 255.),
                        alpha: 0.0,
                    }
                    .into(),
                ),
                order: 2,
                ..default()
            },
            MenuNodeMarker,
        ))
        .id();

    let text_font = TextFont {
        font_size: 36.,
        ..default()
    };
    let text_color = HEADER_TEXT_COLOR.clone();

    commands
        .spawn((
            Node {
                width: Val::Px(1920.0),
                height: Val::Px(1080.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::SpaceAround,
                align_items: AlignItems::Center,
                justify_items: JustifyItems::Center,
                ..default()
            },
            UiTargetCamera(ui_camera_entity),
            MenuNodeMarker,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Percent(50.),
                        height: Val::Percent(50.),
                        position_type: PositionType::Absolute,
                        // justify_content: JustifyContent::SpaceAround,
                        align_items: AlignItems::Center,
                        justify_items: JustifyItems::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(
                        Srgba {
                            red: (30. / 255.),
                            green: (30. / 255.),
                            blue: (46. / 255.),
                            alpha: 1.0,
                        }
                        .into(),
                    ),
                ))
                .with_children(|parent| {
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(12.5),
                            justify_content: JustifyContent::SpaceAround,
                            align_items: AlignItems::Center,
                            justify_items: JustifyItems::Center,

                            ..default()
                        },))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new(""),
                                text_font.clone(),
                                text_color.clone(),
                                OldDevMarker,
                            ));
                        });
                    parent
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(12.5),
                            justify_content: JustifyContent::SpaceAround,
                            align_items: AlignItems::Center,
                            justify_items: JustifyItems::Center,

                            ..default()
                        },))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Dont-Change"),
                                text_font.clone(),
                                text_color.clone(),
                                NewChannelMarker,
                            ));
                        });

                    for i in 0..6 {
                        parent
                            .spawn((Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(12.5),
                                justify_content: JustifyContent::SpaceAround,
                                align_items: AlignItems::Center,
                                justify_items: JustifyItems::Center,

                                ..default()
                            },))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new("---"),
                                    text_font.clone(),
                                    text_color.clone(),
                                    NewDevNameMarker(i),
                                ));
                            });
                    }
                });
        });
}

fn set_midi_dev(mut tracks: Query<(&mut Track, &TrackID)>, new_dev_info: Res<Screen>) {
    let mut tracks: Vec<(_, &TrackID)> = tracks.iter_mut().collect();
    tracks.sort_by_key(|(_track, id)| id.id);

    let Screen::EditTrackMidiDev {
        track,
        dev: midi_dev,
        chan: midi_chan,
    } = new_dev_info.to_owned()
    else {
        return;
    };

    match tracks[track].0.as_mut() {
        Track::Midi {
            steps: _,
            dev,
            chan,
        } => {
            if let Some(new_dev) = midi_dev {
                *dev = new_dev;
            }

            if let Some(new_chan) = midi_chan {
                *chan = new_chan;
            }
        }
        _ => {}
    };
}
