use crate::{
    COL_W, CellCursorMoved, CellMarker, ColumnId, CursorLocation, DisplayStart, MainState, N_STEPS,
    RowId, Screen, Track, TrackID, TracksScrolled, display_midi_note, down_pressed, left_pressed,
    midi_plugin::{BPQ, SyncPulse, get_step_num},
    playing, right_pressed, up_pressed,
};
use bevy::{color::palettes::css::*, platform::collections::HashMap, prelude::*};

// mod shared;

// const SCALE_TIME: u64 = 400;
//
// #[derive(Resource)]
// struct TargetScale {
//     start_scale: f32,
//     target_scale: f32,
//     target_time: Timer,
// }
//
// impl TargetScale {
//     fn set_scale(&mut self, scale: f32) {
//         self.start_scale = self.current_scale();
//         self.target_scale = scale;
//         self.target_time.reset();
//     }
//
//     fn current_scale(&self) -> f32 {
//         let completion = self.target_time.fraction();
//         let t = ease_in_expo(completion);
//         self.start_scale.lerp(self.target_scale, t)
//     }
//
//     fn tick(&mut self, delta: Duration) -> &Self {
//         self.target_time.tick(delta);
//         self
//     }
//
//     fn already_completed(&self) -> bool {
//         self.target_time.finished() && !self.target_time.just_finished()
//     }
// }
//

// #[derive(Debug, Resource, Deref, DerefMut)]
// struct BackgroundColor(Color);
//
// impl Default for BackgroundColor {
//     fn default() -> Self {
//         BackgroundColor(Color::Rgb(
//             ((30. / 255.) * 256.) as u8,
//             ((30. / 255.) * 256.) as u8,
//             ((46. / 255.) * 256.) as u8,
//         ))
//     }
// }
//
// impl WidgetRef for BackgroundColor {
//     fn render_ref(&self, area: Rect, buf: &mut Buffer) {
//         buf.set_style(area, Style::new().bg(self.0));
//     }
// }

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
        app
            // .insert_resource(TargetScale {
            //     start_scale: 1.0,
            //     target_scale: 1.0,
            //     target_time: Timer::new(Duration::from_millis(SCALE_TIME), TimerMode::Once),
            // })
            // .add_systems(OnEnter(MainState::Edit), setup)
            // .add_systems(Update, draw_system)
            // .add_systems(
            //     Update,
            //     (change_scaling, apply_scaling.after(change_scaling)),
            // );
            // .init_resource::<BackgroundColor>()
            .add_event::<CellCursorMoved>()
            .add_event::<TracksScrolled>()
            // .add_systems(OnEnter(MainState::Edit), setup)
            .add_systems(OnEnter(MainState::Edit), (setup, redraw_notes).chain())
            .add_systems(
                Update,
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
                        (
                            redraw_notes.run_if(on_event::<CellCursorMoved>),
                            redraw_notes.run_if(on_event::<TracksScrolled>),
                            // when the step chagnes
                            // redraw_display.run_if(on_event::<TracksScrolled>),
                            // when a note is changed, added, deleated
                            // redraw_display.run_if(on_event::<TracksScrolled>),
                            // redraw_display.run_if(on_event::<TracksScrolled>),
                            // redraw_display.run_if(on_event::<TracksScrolled>),
                        )
                            .run_if(playing),
                    )
                        .run_if(in_main_screen)
                        .run_if(not(mod_key_pressed)),
                    // ui_system,
                    display_step,
                    display_cursor,
                ),
            );
    }
}

// fn setup(mut commands: Commands) {
//     commands.spawn(Camera2d);
// }

// fn setup(mut evs: EventWriter<ResizeEvent>) {
//     evs.write(ResizeEvent(Size {
//         width: COL_W as u16 * 5 + 3,
//         height: N_STEPS as u16 + 2 + 3,
//     }));
// }

// fn ui_system(
//     mut context: ResMut<RatatuiContext>,
//     // frame_count: Res<FrameCount>,
//     // counter: Res<Counter>,
//     // app_state: Res<State<AppState>>,
//     bg_color: Res<BackgroundColor>,
//     // flags: Res<shared::Flags>,
//     // diagnostics: Res<DiagnosticsStore>,
//     // kitty_enabled: Option<Res<KittyEnabled>>,
// ) {
//     context
//         .draw(|frame| {
//             // let area = shared::debug_frame(frame, &flags, &diagnostics, kitty_enabled.as_deref());
//             let area = frame.area();
//
//             // camera_widget.render(area, frame.buffer_mut());
//             let frame_count = Line::from("hello world").right_aligned();
//             frame.render_widget(bg_color.as_ref(), area);
//             frame.render_widget(frame_count, area);
//             // frame.render_widget(counter.as_ref(), area);
//             // frame.render_widget(app_state.get(), area)
//         })
//         .unwrap();
//
//     // Ok(())
// }

/// checks if on the main screen.
pub fn in_main_screen(screen: Res<Screen>) -> bool {
    *screen == Screen::MainScreen
}

/// returns true if the mod key is pressed, indicating that the selected cell should be altered.
pub fn mod_key_pressed(inputs: Query<&Gamepad>) -> bool {
    let mut pressed = false;

    for input in inputs {
        pressed = pressed || input.pressed(GamepadButton::South);
    }

    pressed
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
        font_size: 24.,
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
            justify_content: JustifyContent::SpaceAround,
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
                                            Text::new("Line"),
                                            text_font.clone(),
                                            HEADER_TEXT_COLOR.clone(),
                                        ));
                                        parent.spawn((
                                            Text::new("Note"),
                                            text_font.clone(),
                                            HEADER_TEXT_COLOR.clone(),
                                        ));
                                        parent.spawn((
                                            Text::new("Cmd-1"),
                                            text_font.clone(),
                                            HEADER_TEXT_COLOR.clone(),
                                        ));
                                        parent.spawn((
                                            Text::new("Cmd-2"),
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
                                            100. / 255.,
                                        ))));
                                    } else {
                                        entity.insert(BackgroundColor(Color::Srgba(Srgba::new(
                                            127. / 255.,
                                            132. / 255.,
                                            156. / 255.,
                                            100. / 255.,
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
                                                    // CellMarker {
                                                    //     displayed_track_idx: col_i,
                                                    //     column: 2,
                                                    //     row: i,
                                                    // },
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

// /// System that changes the scale of the ui when pressing up or down on the keyboard.
// fn change_scaling(input: Res<ButtonInput<KeyCode>>, mut ui_scale: ResMut<TargetScale>) {
//     if input.just_pressed(KeyCode::ArrowUp) {
//         let scale = (ui_scale.target_scale * 2.0).min(8.);
//         ui_scale.set_scale(scale);
//         // info!("Scaling up! Scale: {}", ui_scale.target_scale);
//     }
//     if input.just_pressed(KeyCode::ArrowDown) {
//         let scale = (ui_scale.target_scale / 2.0).max(1. / 8.);
//         ui_scale.set_scale(scale);
//         // info!("Scaling down! Scale: {}", ui_scale.target_scale);
//     }
// }
//
// fn apply_scaling(
//     time: Res<Time>,
//     mut target_scale: ResMut<TargetScale>,
//     mut ui_scale: ResMut<UiScale>,
// ) {
//     if target_scale.tick(time.delta()).already_completed() {
//         return;
//     }
//
//     ui_scale.0 = target_scale.current_scale();
// }

fn ease_in_expo(x: f32) -> f32 {
    if x == 0. {
        0.
    } else {
        ops::powf(2.0f32, 5. * x - 5.)
    }
}
