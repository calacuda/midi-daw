use bevy::{
    pbr::wireframe::{NoWireframe, WireframeColor},
    prelude::*,
};
use noise::{NoiseFn, Perlin};
use rand::Rng;
use std::f32::consts::{PI, TAU};

use crate::MainState;

#[derive(Clone, Copy, Debug, Resource)]
pub struct PerlinWrapper(Perlin);

#[derive(Clone, Copy, Debug, Component)]
pub struct UndulateSphere;

#[derive(Clone, Copy, Debug, Component)]
pub struct BaseSphere;

#[derive(Clone, Copy, Debug, Component)]
pub struct BoundingSphere;

#[derive(Component, Deref, DerefMut)]
pub struct UndulateTimer(Timer);

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Zoom(f32);

// Define a component to designate a rotation speed to an entity.
#[derive(Component, Clone, Copy, Debug)]
struct Rotatable {
    speed: f32,
}

#[derive(Default)]
pub struct SphereMode;

impl Plugin for SphereMode {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MainState::Edit),
            (
                gen_perlin,
                add_sphere,
                camera_setup,
                timer_setup,
                tui_camera_setup,
            ),
        )
        // .add_systems(OnExit(Mode::Sphere), camera_teardown)
        .add_systems(Update, (undulate_sphere, rotate_sphere));
    }
}

// fn should_undulate(time: Res<Time>, mut timer: Single<&mut UndulateTimer>) -> bool {
//     timer.tick(time.delta()).just_finished()
// }

fn timer_setup(mut commands: Commands) {
    // Add an entity to the world with a timer
    commands.spawn(UndulateTimer(Timer::from_seconds(
        1.0 / 2.0,
        TimerMode::Repeating,
    )));
}

fn tui_camera_setup(tui_cams: Query<&mut Camera, With<Camera2d>>) {
    for mut cam in tui_cams {
        cam.clear_color = ClearColorConfig::Custom(Color::srgba(0., 0., 0., 0.));
        cam.order += 1;
        info!("setting");
    }
    // info!("done");
}

fn camera_setup(mut commands: Commands) {
    // for mut cam in tui_cams {
    //     cam.clear_color = ClearColorConfig::Custom(Color::srgba(0., 0., 0., 0.));
    //     cam.order += 1;
    //     info!("setting");
    // }
    // info!("done");

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 4.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        Camera {
            clear_color: ClearColorConfig::Custom(
                Srgba {
                    red: (30. / 255.),
                    green: (30. / 255.),
                    blue: (46. / 255.),
                    alpha: 1.0,
                }
                .into(),
            ),
            // order: 1,
            ..default()
        },
        // RatatuiCamera::default(),
        // RatatuiCameraStrategy::luminance_braille(),
        Projection::Perspective(PerspectiveProjection {
            // far: 1_000.0,
            far: 1_000_000.0,
            ..default()
        }),
    ));

    let intensity = 10_000_000.0;
    let light = PointLight {
        shadows_enabled: true,
        // intensity: 10_000_000.,
        intensity,
        range: 1_000_000.0,
        shadow_depth_bias: 0.2,
        radius: PI * 0.5,
        ..default()
    };

    commands.spawn((
        light,
        // Transform::from_xyz(8.0, 16.0, 8.0),
        Transform::from_xyz(1.0, 1.0, 8.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));
}

// fn camera_teardown(
//     mut cmds: Commands,
//     cameras: Query<Entity, With<Camera>>,
//     lights: Query<Entity, With<PointLight>>,
// ) {
//     for camera in cameras.iter() {
//         cmds.entity(camera).despawn_recursive()
//     }
//
//     for light in lights.iter() {
//         cmds.entity(light).despawn_recursive()
//     }
// }

fn gen_perlin(mut cmd: Commands) {
    cmd.insert_resource(PerlinWrapper(Perlin::new(rand::rng().random())));
}

fn add_sphere(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // debug_material: Single<&DebugTexture>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // let mut sphere = |mul| meshes.add(Sphere::default());
    let mut sphere = |mul| {
        meshes.add({
            let mut sphere = Sphere::default();
            sphere.radius *= mul;
            sphere
        })
    };

    cmds.insert_resource(Zoom::default());

    cmds.spawn((
        Mesh3d(sphere(1.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        NoWireframe,
        BaseSphere,
    ));
    cmds.spawn((
        Mesh3d(sphere(1.25)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        MeshMaterial3d(
            materials.add(StandardMaterial::from_color(Color::Srgba(Srgba::rgba_u8(
                // 32, 96, 127, 16,
                // 166, 227, 161, 8,
                // 243, 139, 168, 8,
                147, 153, 178, 8,
            )))),
        ),
        // NoWireframe,
        WireframeColor {
            // color: Color::Srgba(Srgba::rgba_u8(32, 96, 127, 32)),
            // color: Color::Srgba(Srgba::rgba_u8(166, 227, 161, 32)),
            // color: Color::Srgba(Srgba::rgba_u8(243, 139, 168, 32)),
            // color: Color::Srgba(Srgba::rgba_u8(17, 17, 27, 32)),
            // color: Color::Srgba(Srgba::rgba_u8(116, 199, 236, 32)),
            // color: Color::Srgba(Srgba::rgba_u8(203, 166, 247, 32)),
            color: Color::Srgba(Srgba::rgba_u8(24, 24, 37, 32)),
        },
        Rotatable { speed: 0.03125 },
        BoundingSphere,
    ));
    cmds.spawn((
        Mesh3d(sphere(1.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        // MeshMaterial3d(debug_material.0.clone()),
        MeshMaterial3d(
            materials.add(StandardMaterial::from_color(Color::Srgba(Srgba::rgba_u8(
                // 116, 199, 236, 64,
                // 166, 227, 161, 64,
                // 203, 166, 247, 64,
                // 166, 227, 161, 64,
                // 243, 139, 168, 127,
                250, 179, 135, 255,
            )))),
        ),
        NoWireframe,
        Rotatable { speed: 0.03125 },
        UndulateSphere,
    ));
}

fn undulate_sphere(
    sphere: Single<&Mesh3d, With<UndulateSphere>>,
    base_sphere: Single<&Mesh3d, With<BaseSphere>>,
    mut meshes: ResMut<Assets<Mesh>>,
    noise: ResMut<PerlinWrapper>,
    time: Res<Time>,
    mut zoom: ResMut<Zoom>,
    // mut timer: Single<&mut UndulateTimer>,
) {
    // if timer.tick(time.delta()).just_finished() {
    // for ref mut point in sphere.0.
    let Some(base_positions) = meshes.get(base_sphere.0.id()).map(|mesh| {
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
            .to_vec()
    }) else {
        return;
    };

    let Some(mesh) = meshes.get_mut(sphere.0.id()) else {
        return;
    };

    // Get the current mesh data.
    let positions: Vec<[f32; 3]> = mesh
            .attribute_mut(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap()
            .to_vec()
            // .as_typed()
            // .into()
        ;
    // let base_positions: Vec<[f32; 3]> = base_mesh
    //     .attribute_mut(Mesh::ATTRIBUTE_POSITION)
    //     .unwrap()
    //     .as_float3()
    //     .unwrap()
    //     .to_vec()
    // .as_typed()
    // .into()

    // Modify the positions. For example, scale the mesh:
    // let scale_factor = 2.0;
    let mut new_positions: Vec<[f32; 3]> = Vec::with_capacity(positions.len());
    let td = time.delta().as_secs_f32();
    zoom.0 += 0.5 * TAU * td;

    for pos in base_positions.iter() {
        let scale_factor = noise.0.get([
            (pos[0] * zoom.0) as f64,
            (pos[1] * zoom.0) as f64,
            (pos[2] * zoom.0) as f64,
        ]) as f32
            * 0.25;

        new_positions.push([
            pos[0] + pos[0] * scale_factor,
            pos[1] + pos[1] * scale_factor,
            pos[2] + pos[2] * scale_factor,
        ]);
    }

    // Update the mesh with the modified positions.
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_positions);

    // info!("undulated");

    // Optionally, re-compute normals after modifying positions.
    // mesh.compute_flat_normals(); // Or mesh.compute_smooth_normals();
    mesh.compute_smooth_normals();
    // mesh.verte

    // You can also add or remove attributes, change indices, etc. as needed.
    // }
}

// This system will rotate any entity in the scene with a Rotatable component around its y-axis.
fn rotate_sphere(mut spheres: Query<(&mut Transform, &Rotatable)>, timer: Res<Time>) {
    for (mut transform, sphere) in &mut spheres {
        // The speed is first multiplied by TAU which is a full rotation (360deg) in radians,
        // and then multiplied by delta_secs which is the time that passed last frame.
        // In other words. Speed is equal to the amount of rotations per second.
        transform.rotate_y(sphere.speed * TAU * timer.delta_secs());
    }
}
