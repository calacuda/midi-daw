use crate::{
    CellCursorMoved, CellMarker, ColumnId, CursorLocation, DisplayStart, MainState, N_STEPS,
    NoteChanged, RowId, ScreenState, Track, TrackID, TracksScrolled, a_and_b_pressed,
    display::midi_assign::MidiAssignmentPlugin,
    display_midi_note, down_pressed, left_pressed,
    midi_plugin::{BPQ, SyncPulse, get_step_num, on_thirtysecond_note},
    playing, right_pressed, up_pressed,
};
use bevy::{color::palettes::css::*, platform::collections::HashMap, prelude::*};

pub mod midi_assign;

const TEXT_COLOR: TextColor = TextColor(Color::Srgba(Srgba::new(
    17. / 255.,
    17. / 255.,
    17. / 255.,
    1.0,
)));

const STEP_TEXT_COLOR: TextColor = TextColor(Color::Srgba(Srgba::new(
    243. / 255.,
    139. / 255.,
    168. / 255.,
    1.0,
)));

const HEADER_TEXT_COLOR: TextColor = TextColor(Color::Srgba(Srgba::new(
    166. / 255.,
    227. / 255.,
    161. / 255.,
    1.0,
)));

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct TrackTargetMarker;

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct LineNumMarker;

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct NoteDisplayMarker;

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct Cmd1DisplayMarker;

#[derive(Component, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct Cmd2DisplayMarker;

pub struct MainDisplayPlugin;

impl Plugin for MainDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MidiAssignmentPlugin)
            .add_event::<CellCursorMoved>()
            .add_event::<TracksScrolled>()
            .add_event::<NoteChanged>()
            .add_systems(Startup, apply_scaling)
            .add_systems(OnEnter(MainState::Edit), (setup, redraw_notes).chain())
            .add_systems(
                Update,
                (
                    (
                        (
                            (
                                scroll_left.run_if(cursor_at_min_x).run_if(left_pressed),
                                scroll_right.run_if(cursor_at_max_x).run_if(right_pressed),
                            )
                                .run_if(more_than_four_tracks),
                            move_cursor_left
                                .run_if(not(cursor_at_min_x))
                                .run_if(left_pressed),
                            move_cursor_right
                                .run_if(not(cursor_at_max_x))
                                .run_if(right_pressed),
                            move_cursor_up
                                .run_if(not(cursor_at_min_y))
                                .run_if(up_pressed),
                            move_cursor_down
                                .run_if(not(cursor_at_max_y))
                                .run_if(down_pressed),
                        )
                            .run_if(not(mod_key_pressed)),
                        display_cursor,
                        (
                            redraw_notes.run_if(on_event::<CellCursorMoved>),
                            redraw_notes.run_if(on_event::<TracksScrolled>),
                            // when the step chagnes
                            redraw_notes.run_if(on_event::<NoteChanged>),
                        ),
                        // .run_if(playing),
                        (
                            (
                                small_increment_note.run_if(up_pressed),
                                small_decrement_note.run_if(down_pressed),
                                big_increment_note.run_if(right_pressed),
                                big_decrement_note.run_if(left_pressed),
                                delete_note.run_if(a_and_b_pressed),
                            )
                                .run_if(note_selected),
                            // TODO: cmd editing
                            // (
                            //     small_increment_note.run_if(up_pressed),
                            //     small_decrement_note.run_if(down_pressed),
                            //     big_increment_note.run_if(right_pressed),
                            //     big_decrement_note.run_if(left_pressed),
                            // )
                            //     .run_if(not(note_selected)),
                        )
                            .run_if(mod_key_pressed),
                    )
                        // .run_if(in_main_screen),
                        .run_if(in_state(ScreenState::MainScreen)),
                    display_step.run_if(playing.and(on_thirtysecond_note)),
                ),
            );
    }
}

// /// checks if on the main screen.
// pub fn in_main_screen(screen: Res<Screen>) -> bool {
//     *screen == Screen::MainScreen
// }

/// returns true if the mod key is pressed, indicating that the selected cell should be altered.
pub fn mod_key_pressed(inputs: Query<&Gamepad>) -> bool {
    let mut pressed = false;

    for input in inputs {
        pressed = pressed || input.pressed(GamepadButton::South);
    }

    pressed
}

fn note_selected(cursor: Res<CursorLocation>) -> bool {
    (cursor.1 % 3) == 0
}

fn cursor_at_min_y(cursor: Res<CursorLocation>) -> bool {
    cursor.0 == 0
}

fn cursor_at_max_y(cursor: Res<CursorLocation>) -> bool {
    cursor.0 == N_STEPS
}

fn cursor_at_min_x(cursor: Res<CursorLocation>) -> bool {
    cursor.1 == 0
}

fn cursor_at_max_x(cursor: Res<CursorLocation>) -> bool {
    // n_tracks displayed at once * n_columns per track
    cursor.1 == 3 * 4
}

fn small_increment_note(
    // notes: Query<(&mut Text, &ColumnId, &RowId), With<NoteDisplayMarker>>,
    display_start: Res<DisplayStart>,
    mut tracks: Query<(&mut Track, &TrackID)>,
    cursor: Res<CursorLocation>,
    mut evs: EventWriter<NoteChanged>,
) {
    let mut tracks: Vec<(_, &TrackID)> = tracks.iter_mut().collect();
    tracks.sort_by_key(|(_track, id)| id.id);

    let start = display_start.0;
    let col = cursor.1 / 3;
    let track_num = start + col;

    // warn!("incrementing");

    match tracks[track_num].0.as_mut() {
        Track::Midi {
            steps,
            dev: _,
            chan: _,
        } => {
            let new_note = steps[cursor.0]
                .note
                .map(|note| (note + 1) % 127)
                .unwrap_or(0);
            steps[cursor.0].note.replace(new_note);
        }
        _ => {}
    };

    evs.write(NoteChanged::SmallUp);
}

fn small_decrement_note(
    // notes: Query<(&mut Text, &ColumnId, &RowId), With<NoteDisplayMarker>>,
    display_start: Res<DisplayStart>,
    mut tracks: Query<(&mut Track, &TrackID)>,
    cursor: Res<CursorLocation>,
    mut evs: EventWriter<NoteChanged>,
) {
    let mut tracks: Vec<(_, &TrackID)> = tracks.iter_mut().collect();
    tracks.sort_by_key(|(_track, id)| id.id);

    let start = display_start.0;
    let col = cursor.1 / 3;
    let track_num = start + col;

    // warn!("incrementing");

    match tracks[track_num].0.as_mut() {
        Track::Midi {
            steps,
            dev: _,
            chan: _,
        } => {
            let new_note = steps[cursor.0]
                .note
                .map(|note| if note > 0 { note - 1 } else { 126 })
                .unwrap_or(0);
            steps[cursor.0].note.replace(new_note);
        }
        _ => {}
    };

    evs.write(NoteChanged::SmallDown);
}

fn big_increment_note(
    // notes: Query<(&mut Text, &ColumnId, &RowId), With<NoteDisplayMarker>>,
    display_start: Res<DisplayStart>,
    mut tracks: Query<(&mut Track, &TrackID)>,
    cursor: Res<CursorLocation>,
    mut evs: EventWriter<NoteChanged>,
) {
    let mut tracks: Vec<(_, &TrackID)> = tracks.iter_mut().collect();
    tracks.sort_by_key(|(_track, id)| id.id);

    let start = display_start.0;
    let col = cursor.1 / 3;
    let track_num = start + col;

    // warn!("incrementing");

    match tracks[track_num].0.as_mut() {
        Track::Midi {
            steps,
            dev: _,
            chan: _,
        } => {
            let new_note = steps[cursor.0]
                .note
                .map(|note| (note + 12) % 127)
                .unwrap_or(0);
            steps[cursor.0].note.replace(new_note);
        }
        _ => {}
    };

    evs.write(NoteChanged::BigUp);
}

fn big_decrement_note(
    // notes: Query<(&mut Text, &ColumnId, &RowId), With<NoteDisplayMarker>>,
    display_start: Res<DisplayStart>,
    mut tracks: Query<(&mut Track, &TrackID)>,
    cursor: Res<CursorLocation>,
    mut evs: EventWriter<NoteChanged>,
) {
    let mut tracks: Vec<(_, &TrackID)> = tracks.iter_mut().collect();
    tracks.sort_by_key(|(_track, id)| id.id);

    let start = display_start.0;
    let col = cursor.1 / 3;
    let track_num = start + col;

    // warn!("incrementing");

    match tracks[track_num].0.as_mut() {
        Track::Midi {
            steps,
            dev: _,
            chan: _,
        } => {
            let new_note = steps[cursor.0]
                .note
                .map(|note| if note > 11 { note - 12 } else { 114 })
                .unwrap_or(0);
            steps[cursor.0].note.replace(new_note);
        }
        _ => {}
    };

    evs.write(NoteChanged::BigDown);
}

fn delete_note(
    // notes: Query<(&mut Text, &ColumnId, &RowId), With<NoteDisplayMarker>>,
    display_start: Res<DisplayStart>,
    mut tracks: Query<(&mut Track, &TrackID)>,
    cursor: Res<CursorLocation>,
    mut evs: EventWriter<NoteChanged>,
) {
    let mut tracks: Vec<(_, &TrackID)> = tracks.iter_mut().collect();
    tracks.sort_by_key(|(_track, id)| id.id);

    let start = display_start.0;
    let col = cursor.1 / 3;
    let track_num = start + col;

    // warn!("incrementing");

    match tracks[track_num].0.as_mut() {
        Track::Midi {
            steps,
            dev: _,
            chan: _,
        } => {
            let new_note = None;
            steps[cursor.0].note = new_note;
        }
        _ => {}
    };

    evs.write(NoteChanged::Deleted);
}

fn scroll_left(
    // button_inputs: Res<ButtonInput<GamepadButton>>,
    // location: Res<CursorLocation>,
    mut display_start: ResMut<DisplayStart>,
    mut scroll_ev: EventWriter<TracksScrolled>,
) {
    if display_start.0 > 0 {
        display_start.0 -= 1;
        scroll_ev.write(TracksScrolled::Left);
    }
}

fn scroll_right(
    mut display_start: ResMut<DisplayStart>,
    tracks: Query<&Track>,
    mut scroll_ev: EventWriter<TracksScrolled>,
) {
    let n_tracks = tracks.iter().len();

    if display_start.0 < n_tracks - 3 {
        display_start.0 += 1;
        scroll_ev.write(TracksScrolled::Right);
    }
}

fn more_than_four_tracks(tracks: Query<&Track>) -> bool {
    let n_tracks = tracks.iter().len();

    n_tracks > 4
}

fn move_cursor_up(mut cursor: ResMut<CursorLocation>, mut cursor_ev: EventWriter<CellCursorMoved>) {
    cursor.0 -= 1;
    cursor_ev.write(CellCursorMoved::Up);
}

fn move_cursor_down(
    mut cursor: ResMut<CursorLocation>,
    mut cursor_ev: EventWriter<CellCursorMoved>,
) {
    cursor.0 += 1;
    cursor.0 %= N_STEPS;
    cursor_ev.write(CellCursorMoved::Down);
}

fn move_cursor_left(
    mut cursor: ResMut<CursorLocation>,
    mut cursor_ev: EventWriter<CellCursorMoved>,
) {
    cursor.1 -= 1;
    cursor_ev.write(CellCursorMoved::Left);
}

fn move_cursor_right(
    mut cursor: ResMut<CursorLocation>,
    mut cursor_ev: EventWriter<CellCursorMoved>,
) {
    cursor.1 += 1;
    cursor.1 %= 3 * 4;
    cursor_ev.write(CellCursorMoved::Right);
}

fn redraw_notes(
    notes: Query<(&mut Text, &ColumnId, &RowId), With<NoteDisplayMarker>>,
    display_start: Res<DisplayStart>,
    tracks: Query<(&Track, &TrackID)>,
) {
    let mut tracks: Vec<(&Track, &TrackID)> = tracks.into_iter().collect();
    tracks.sort_by_key(|(_track, id): &(&Track, &TrackID)| id.id);

    let mut notes_text: HashMap<(usize, usize), Mut<'_, Text>> = notes
        .into_iter()
        .map(|(text, col, row)| ((col.0, row.0), text))
        .collect();

    let start = display_start.0;
    let end = start + std::cmp::min(4, tracks.len());

    for (i, track_i) in (start..end).enumerate() {
        let track = tracks[track_i];
        match track.0 {
            Track::Midi {
                steps,
                dev: _,
                chan: _,
            } => {
                for (step_i, step) in steps.iter().enumerate() {
                    if let Some(text) = notes_text.get_mut(&(i, step_i)) {
                        if let Some(note_text) = step.note.map(display_midi_note) {
                            text.0 = note_text;
                        } else {
                            text.0 = "---".into();
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// changes the color of the step lable that is being played
fn display_step(
    line_num: Query<(&mut TextColor, &Text), With<LineNumMarker>>,
    pulse: Res<SyncPulse>,
    bpq: Res<BPQ>,
) {
    let step_i = get_step_num(&pulse, &bpq);
    let target = format!("{:0>2}: ", step_i + 1);

    for (mut color, text) in line_num {
        // if text.0.ends_with(format!("{}", step_i)) && color.clone() == TEXT_COLOR {
        if text.0 == target && color.clone() == TEXT_COLOR {
            // info!("displaying step");
            *color = STEP_TEXT_COLOR;
        } else if text.0 != target && color.clone() == STEP_TEXT_COLOR {
            *color = TEXT_COLOR;
        }
    }
}

/// changes the color of the step lable that is being played
fn display_cursor(
    mut commands: Commands,
    cells: Query<(Entity, &CellMarker)>,
    cursor: Res<CursorLocation>,
) {
    let track = cursor.1 / 3;
    let col = cursor.1 % 3;
    let row = cursor.0;

    for (entity, cell_mark) in cells {
        // info!("drawing at: {cursor:?} => ({track}, {col}, {row})");

        if cell_mark.displayed_track_idx == track && cell_mark.column == col && cell_mark.row == row
        {
            // warn!("drawing at: {cursor:?} => ({track}, {col}, {row})");
            // set boarder to cursor color
            commands
                .entity(entity)
                // 116, 199, 236
                .insert(Outline::new(
                    Val::Px(10.),
                    Val::Px(-5.),
                    // Val::ZERO,
                    Color::Srgba(Srgba {
                        red: 116. / 255.,
                        green: 199. / 255.,
                        blue: 236. / 255.,
                        alpha: 1.,
                    }),
                ));
            // commands.entity(entity).remove::<BorderColor>();
        } else {
            // rm boarder from
            commands.entity(entity).remove::<Outline>();
            // commands.entity(entity).remove::<BorderColor>();
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
    ));

    let text_font = TextFont {
        font_size: 36.,
        ..default()
    };
    // let text_color = TextColor(Srgba::new(205. / 255., 214. / 255., 244. / 255., 1.0).into());
    // let text_color = TextColor(Srgba::new(17. / 255., 17. / 255., 17. / 255., 1.0).into());
    let text_color = TEXT_COLOR.clone();

    commands
        .spawn((Node {
            width: Val::Px(1920.0),
            height: Val::Px(1080.0),
            position_type: PositionType::Absolute,
            // justify_content: JustifyContent::SpaceAround,
            align_items: AlignItems::Center,
            justify_items: JustifyItems::Center,
            ..default()
        },))
        .with_children(|parent| {
            parent
                .spawn((Node {
                    width: Val::Percent(80.0),
                    height: Val::Percent(100.0),
                    ..default()
                },))
                .with_children(|parent| {
                    for col_i in 0..4 {
                        parent
                            .spawn((
                                Node {
                                    width: Val::Percent(25.0),
                                    height: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::SpaceAround,
                                    ..default()
                                },
                                ColumnId(col_i),
                            ))
                            .with_children(|parent| {
                                let cell_h = 100.0 / (N_STEPS + 6) as f32;
                                parent
                                    .spawn((
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(cell_h * 3.0),
                                            flex_direction: FlexDirection::Row,
                                            justify_content: JustifyContent::SpaceAround,
                                            ..default()
                                        },
                                        // RowId(i),
                                    ))
                                    .with_children(|parent| {
                                        // TODO: add: a friendly track name display here
                                        // parent.spawn((
                                        //     Text::new(format!("Track => {i}")),
                                        //     text_font.clone(),
                                        //     TextColor::BLACK,
                                        //     TrackLable,
                                        // ));
                                        parent.spawn((
                                            Text::new(""),
                                            text_font.clone(),
                                            text_color.clone(),
                                            TrackTargetMarker,
                                        ));
                                        parent.spawn((
                                            Text::new("\n"),
                                            text_font.clone(),
                                            text_color.clone(),
                                        ));
                                    });

                                parent
                                    .spawn((
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(cell_h),
                                            flex_direction: FlexDirection::Row,
                                            justify_content: JustifyContent::SpaceAround,
                                            ..default()
                                        },
                                        // RowId(i),
                                    ))
                                    .with_children(|parent| {
                                        if col_i > 0 {
                                            parent.spawn((
                                                Text::new("|"),
                                                text_font.clone(),
                                                text_color.clone(),
                                            ));
                                        }

                                        parent.spawn((
                                            Text::new("Ln"),
                                            text_font.clone(),
                                            HEADER_TEXT_COLOR.clone(),
                                        ));
                                        parent.spawn((
                                            Text::new("Note"),
                                            text_font.clone(),
                                            HEADER_TEXT_COLOR.clone(),
                                        ));
                                        parent.spawn((
                                            Text::new("Cmd"),
                                            text_font.clone(),
                                            HEADER_TEXT_COLOR.clone(),
                                        ));
                                        parent.spawn((
                                            Text::new("Cmd"),
                                            text_font.clone(),
                                            HEADER_TEXT_COLOR.clone(),
                                        ));
                                    });

                                for i in 0..N_STEPS {
                                    let mut entity = parent.spawn((
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(cell_h),
                                            flex_direction: FlexDirection::Row,
                                            justify_content: JustifyContent::SpaceAround,
                                            ..default()
                                        },
                                        RowId(i),
                                    ));

                                    if i % 2 == 0 {
                                        entity.insert(BackgroundColor(Color::Srgba(Srgba::new(
                                            88. / 255.,
                                            91. / 255.,
                                            112. / 255.,
                                            128. / 255.,
                                        ))));
                                    } else {
                                        entity.insert(BackgroundColor(Color::Srgba(Srgba::new(
                                            127. / 255.,
                                            132. / 255.,
                                            156. / 255.,
                                            128. / 255.,
                                        ))));
                                    }

                                    entity.with_children(|parent| {
                                        if col_i > 0 {
                                            parent.spawn((
                                                Text::new("|"),
                                                text_font.clone(),
                                                text_color.clone(),
                                            ));
                                        }

                                        parent.spawn((
                                            Text::new(format!("{:0>2}: ", i + 1)),
                                            text_font.clone(),
                                            text_color.clone(),
                                            LineNumMarker,
                                        ));
                                        parent
                                            .spawn((
                                                Node {
                                                    width: Val::Percent(25.0),
                                                    // height: Val::Percent(cell_h),
                                                    justify_content: JustifyContent::SpaceAround,
                                                    ..default()
                                                },
                                                CellMarker {
                                                    displayed_track_idx: col_i,
                                                    column: 0,
                                                    row: i,
                                                },
                                            ))
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    Text::new("---"),
                                                    text_font.clone(),
                                                    text_color.clone(),
                                                    NoteDisplayMarker,
                                                    ColumnId(col_i),
                                                    RowId(i),
                                                ));
                                            });
                                        parent
                                            .spawn((
                                                Node {
                                                    width: Val::Percent(25.0),
                                                    // height: Val::Percent(cell_h),
                                                    justify_content: JustifyContent::SpaceAround,
                                                    ..default()
                                                },
                                                CellMarker {
                                                    displayed_track_idx: col_i,
                                                    column: 1,
                                                    row: i,
                                                },
                                            ))
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    Text::new("----"),
                                                    text_font.clone(),
                                                    text_color.clone(),
                                                    Cmd1DisplayMarker,
                                                    ColumnId(col_i),
                                                    RowId(i),
                                                    // CellMarker {
                                                    //     displayed_track_idx: col_i,
                                                    //     column: 1,
                                                    //     row: i,
                                                    // },
                                                ));
                                            });
                                        parent
                                            .spawn((
                                                Node {
                                                    width: Val::Percent(25.0),
                                                    // height: Val::Percent(cell_h),
                                                    justify_content: JustifyContent::SpaceAround,
                                                    ..default()
                                                },
                                                CellMarker {
                                                    displayed_track_idx: col_i,
                                                    column: 2,
                                                    row: i,
                                                },
                                            ))
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    Text::new("----"),
                                                    text_font.clone(),
                                                    text_color.clone(),
                                                    Cmd2DisplayMarker,
                                                    ColumnId(col_i),
                                                    RowId(i),
                                                ));
                                            });
                                    });
                                }
                            });
                    }
                });
            parent.spawn((
                Node {
                    width: Val::Percent(15.0),
                    height: Val::Percent(50.0),
                    ..default()
                },
                BackgroundColor(BLUE.into()),
            ));
        });
}

fn apply_scaling(
    // time: Res<Time>,
    // mut target_scale: ResMut<TargetScale>,
    mut ui_scale: ResMut<UiScale>,
) {
    // if target_scale.tick(time.delta()).already_completed() {
    //     return;
    // }

    ui_scale.0 = 0.5;
}
