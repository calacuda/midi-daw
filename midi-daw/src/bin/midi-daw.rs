use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    window::WindowMode,
};
// use bevy_ratatui::RatatuiPlugins;
use crossbeam::channel::unbounded;
use midi_daw::midi::{MidiDev, dev::new_midi_dev, out::midi_out};
use midi_daw_lib::{
    CursorLocation, DisplayStart, MainState, MidiOutput, N_STEPS, NewMidiDev, Screen, ScreenState,
    Step, Track, TrackID, TrackerCmd, button_tracker::ButtonTrackerPlugin,
    display::MainDisplayPlugin, midi_plugin::MidiOutPlugin, sphere::SphereMode,
};
use midi_msg::Channel;
use std::thread::spawn;

// use bevy_ascii_terminal::{render::TerminalMeshTileScaling, *};

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
        .send(MidiDev::CreateVirtual("Chan-1".into()))
        .unwrap();
    new_midi_dev_tx
        .send(MidiDev::CreateVirtual("Chan-2".into()))
        .unwrap();
    new_midi_dev_tx
        .send(MidiDev::CreateVirtual("Chan-3".into()))
        .unwrap();
    new_midi_dev_tx
        .send(MidiDev::CreateVirtual("Chan-4".into()))
        .unwrap();
    //
    // let frame_time = std::time::Duration::from_secs_f32(1. / 60.);

    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // fit_canvas_to_parent: true,
                        // fullsize_content_view: true,
                        mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest()),
            // RatatuiPlugins {
            //     enable_input_forwarding: true,
            //     enable_kitty_protocol: true,
            //     enable_mouse_capture: true,
            //     ..default()
            // },
            MidiOutPlugin,
            MainDisplayPlugin,
            ButtonTrackerPlugin,
            WireframePlugin::default(),
            SphereMode,
        ))
        .insert_resource(WireframeConfig {
            // The global wireframe config enables drawing of wireframes on every mesh,
            // except those with `NoWireframe`. Meshes with `Wireframe` will always have a wireframe,
            // regardless of the global configuration.
            global: true,
            // Controls the default color of all wireframes. Used as the default color for global wireframes.
            // Can be changed per mesh using the `WireframeColor` component.
            default_color: Srgba {
                red: (166. / 255.),
                green: (227. / 255.),
                blue: (161. / 255.),
                alpha: 1.0,
            }
            .into(),
        })
        .insert_resource(ClearColor(
            Srgba {
                red: (30. / 255.),
                green: (30. / 255.),
                blue: (46. / 255.),
                alpha: 0.25,
            }
            .into(),
        ))
        // .insert_resource(ClearColor(Color::BLACK))
        .init_state::<MainState>()
        .init_state::<ScreenState>()
        .insert_resource(NewMidiDev(new_midi_dev_tx))
        .insert_resource(MidiOutput(midi_msg_out_tx))
        .init_resource::<CursorLocation>()
        .init_resource::<DisplayStart>()
        .init_resource::<Screen>()
        .add_event::<Screen>()
        .add_systems(PreUpdate, keyboard_input_system_windowed)
        .add_systems(Startup, (/* setup, */ setup_tracks, enter_edit_state))
        .run();
}

// fn keyboard_input_system_windowed(
//     keys: Res<ButtonInput<KeyCode>>,
//     mut app_exit: EventWriter<AppExit>,
//     // mut counter_events: EventWriter<CounterEvent>,
// ) {
//     if keys.just_pressed(KeyCode::KeyQ) {
//         app_exit.write_default();
//     }
//     if keys.just_pressed(KeyCode::KeyP) {
//         panic!("Panic!");
//     }
//     // if keys.pressed(KeyCode::ArrowLeft) {
//     //     counter_events.write(CounterEvent::Decrement);
//     // }
//     // if keys.pressed(KeyCode::ArrowRight) {
//     //     counter_events.write(CounterEvent::Increment);
//     // }
// }

// fn setup(mut commands: Commands) {
//     let col_h = N_STEPS + 3;
//
//     // commands.spawn((
//     //     // Terminal::new([12, 1]).with_string([0, 0], "Hello world!".fg(color::BLUE)),
//     //     // Terminal::new([COL_W * 5 + 1, col_h + col_h / 2])
//     //     Terminal::new([COL_W * 5 + 1 + 2, col_h + 2])
//     //         .with_string([0, 0], "Hello world!".fg(color::BLUE).bg(color::RED)),
//     //     TerminalBorder::single_line(),
//     //     // TerminalMeshTileScaling((2.2, 1.0).into()),
//     //     // TerminalFont::Custom("assets/MyFont.png".to_string()),
//     //     TerminalFont::SazaroteCurses12x12,
//     //     // Transform::from_scale(Vec3 {
//     //     //     x: 1.25,
//     //     //     y: 1.25,
//     //     //     z: 0.,
//     //     // }),
//     // ));
//     // commands.spawn((
//     //     TerminalCamera::new(),
//     //     // TerminalMeshTileScaling((1.2, 1.0).into()),
//     //     // Transform::from_xyz(0.0, 0.0, 5.0),
//     //     Projection::Orthographic(OrthographicProjection {
//     //         // near: (),
//     //         // far: (),
//     //         // viewport_origin: (),
//     //         scaling_mode: bevy::render::camera::ScalingMode::FixedVertical {
//     //             viewport_height: 1080. * 2.0,
//     //         },
//     //         // scale: (),
//     //         // area: (),
//     //         ..OrthographicProjection::default_2d()
//     //     }),
//     // ));
// }

fn setup_tracks(mut cmds: Commands) {
    // let steps: Vec<Step<MidiCmd>> = (0..N_STEPS).map(|_| Step::default()).collect();
    let mut steps: Vec<Step> = (0..N_STEPS).map(|_| Step::default()).collect();
    let plain_steps = steps.clone();

    // let notes = [48, 52, 55, 59];
    // for (i, step) in steps.iter_mut().step_by(N_STEPS / 4).enumerate() {
    //     step.note = Some(notes[i % 4]);
    //     step.cmds.0 = TrackerCmd::HoldFor {
    //         notes: 1.try_into().unwrap(),
    //         // notes: 3.try_into().unwrap(),
    //     };
    //     // step.cmds.1 = TrackerCmd::Panic;
    // }
    //
    // steps[0].cmds.1 = TrackerCmd::Chord {
    //     chord: vec![4, 7, 11],
    // };
    // // steps[1].cmds.0 = TrackerCmd::Panic;
    // steps[N_STEPS / 2].cmds.1 = TrackerCmd::Chord {
    //     chord: vec![3, 7, 10],
    // };

    steps[0].note = Some(48);
    steps[0].cmds.0 = TrackerCmd::Chord {
        chord: vec![4, 7, 11],
    };
    steps[0].cmds.1 = TrackerCmd::Roll {
        times: 7.try_into().unwrap(),
    };

    cmds.spawn((
        TrackID {
            id: 0,
            // playing: false,
            playing: true,
        },
        // track,
        // Track::default(),
        Track {
            steps: steps.clone(),
            dev: "Chan-1".into(),
            chan: Channel::Ch1,
        },
    ));
    cmds.spawn((
        TrackID {
            id: 1,
            playing: true,
        },
        // Track::default(),
        Track {
            steps: plain_steps.clone(),
            dev: "Chan-2".into(),
            chan: Channel::Ch1,
        },
    ));
    cmds.spawn((
        TrackID {
            id: 2,
            playing: true,
        },
        // Track::default(),
        Track {
            steps: plain_steps.clone(),
            dev: "Chan-3".into(),
            chan: Channel::Ch1,
        },
    ));
    cmds.spawn((
        TrackID {
            id: 3,
            playing: true,
        },
        // Track::default(),
        Track {
            steps: plain_steps.clone(),
            dev: "Chan-4".into(),
            chan: Channel::Ch1,
        },
    ));
}

fn enter_edit_state(mut main_state: ResMut<NextState<MainState>>) {
    main_state.set(MainState::Edit);
}

// fn setup_cursor(mut cmds: Commands) {
//     cmds.spawn((
//         // TextComponent {
//         //     text: ">".into(),
//         //     point: Point::new(x_from_col(2), row_from_line(2)),
//         //     color: Some(Rgb565::CYAN),
//         //     ..default()
//         // },
//         // CursorText,
//     ));
// }

// fn setup_track_dis(mut cmds: Commands) {}

fn keyboard_input_system_windowed(
    keys: Res<ButtonInput<KeyCode>>,
    mut app_exit: EventWriter<AppExit>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        app_exit.write_default();
    }
    if keys.just_pressed(KeyCode::KeyP) {
        panic!("Panic!");
    }
}
