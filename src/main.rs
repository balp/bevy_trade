use std::f32::consts::PI;

use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use rand::Rng;

#[derive(Component, Debug)]
struct MapPosition {
    pos: Vec3,
}

impl MapPosition {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { pos: Vec3::new(x, y, z) }
    }
}


#[derive(Component, Debug)]
struct WaterLevel {
    depth: f32,
}

impl WaterLevel {
    pub fn new(depth: f32) -> Self {
        Self { depth }
    }
}


fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb_linear(0.05, 0.05, 0.1)))
        .add_startup_system(make_map)
        .add_startup_system(setup)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_system(animate_light_direction)
        .add_plugins(DefaultPlugins)
        .add_plugin(FlyCameraPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn scaling(scale: f32, step_size: usize) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(-scale..scale)
    // (rng.gen_range(0.0..1.0) * 2.0 - 1.0) * (step_size as f32) * scale
}

fn gen_heigth_map() -> [[f32; 129]; 129] {
    const WIDTH: usize = 129;
    let mut height_map: [[f32; WIDTH]; WIDTH] = [[0.0; WIDTH]; WIDTH];
    let mut step_size: usize = 128;
    let mut scale = 1.0;
    // let mut scale = 1.0 / 128.0;
    height_map[0][0] = scaling(scale, step_size);
    height_map[WIDTH - 1][0] = scaling(scale, step_size);
    height_map[0][WIDTH - 1] = scaling(scale, step_size);
    height_map[WIDTH - 1][WIDTH - 1] = scaling(scale, step_size);

    while step_size > 2 {
        let half_step: usize = step_size / 2;

        let step_index = step_size;
        let half_step_index = half_step;

        // Diamond step
        for x in (0..WIDTH - 1).step_by(step_size) {
            for z in (0..WIDTH - 1).step_by(step_size) {
                println!("diamond step: {:?},{:?} {:?} {:?}", x, z, step_size, half_step);

                let top_left = height_map[x][z];
                let top_right = height_map[x + step_index][z];
                let bottom_left = height_map[x][z + step_index];
                let bottom_right = height_map[x + step_index][z + step_index];
                let middle = (top_left + top_right + bottom_right + bottom_left) / 4.0 + scaling(scale, step_size);
                height_map[x + half_step_index][z + half_step_index] = middle;
            }
        }
        // Square Step
        for x in (0..WIDTH - 1).step_by(step_size) {
            for z in (0..WIDTH - 1).step_by(step_size) {
                println!("square step: {:?},{:?} {:?} {:?}", x, z, step_size, half_step);
                let top_left = height_map[x][z];
                let top_right = height_map[x + step_index][z];
                let bottom_left = height_map[x][z + step_index];
                let bottom_right = height_map[x + step_index][z + step_index];
                let middle = height_map[x + half_step][z + half_step];

                height_map[x][z + half_step_index] = (top_left + middle + bottom_left) / 3.0 + scaling(scale, step_size);
                height_map[x + half_step_index][z] = (top_left + middle + top_right) / 3.0 + scaling(scale, step_size);
                height_map[x + step_index][z + step_index] = (top_right + middle + bottom_right) / 3.0 + scaling(scale, step_size);
                height_map[x + half_step_index][z + step_index] = (bottom_left + middle + bottom_right) / 3.0 + scaling(scale, step_size);
            }
        }
        // Prep
        step_size = step_size / 2;
        scale = scale * 1.6;
    }
    for x in 0..WIDTH {
        for z in 0..WIDTH {
            let mut neighbours: Vec<f32> = vec![height_map[x][z]];
            if x > 0 {
                neighbours.push(height_map[x - 1][z]);
            }
            if x < WIDTH - 2 {
                neighbours.push(height_map[x + 1][z]);
            }
            if z > 0 {
                neighbours.push(height_map[x][z - 1]);
            }
            if z < WIDTH - 2 {
                neighbours.push(height_map[x][z + 1]);
            }
            if x > 0 && z > 0 {
                neighbours.push(height_map[x - 1][z - 1]);
            }
            if x < WIDTH - 2 && z > 0 {
                neighbours.push(height_map[x + 1][z - 1]);
            }
            if x > 0 && z < WIDTH - 2 {
                neighbours.push(height_map[x - 1][z + 1]);
            }
            if x < WIDTH - 2 && z < WIDTH - 2 {
                neighbours.push(height_map[x + 1][z + 1]);
            }
            height_map[x][z] = neighbours.iter().sum::<f32>() / neighbours.len() as f32;
        }
    }
    height_map
}


fn make_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Make a random height map
    let height_map: [[f32; 129]; 129] = gen_heigth_map();
    println!("map: {:?}", height_map);
    let mut min = f32::MAX;
    let mut max = f32::MIN;
    for row in height_map {
        for col in row {
            if col < min {
                min = col;
            }
            if col > max {
                max = col;
            }
        }
    }
    println!("min: {} max: {}", min, max);

    for x in 0..128 {
        for z in 0..128 {
            commands.spawn(MapPosition::new(x as f32, height_map[x][z], z as f32));
            commands.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(1.0, height_map[x][z] + f32::abs(min), 1.0))),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(x as f32, min, z as f32),
                ..default()
            });
        }
    };
    commands.spawn(MapPosition::new(64.0, height_map[64][64], 64.0))
        .insert(WaterLevel::new(0.1))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 0.1, 1.0))),
            material: materials.add(Color::rgba(0.1, 0.01, 0.5, 0.5).into()),
            transform: Transform::from_xyz(64.0, height_map[64][64], 64.0),
            ..default()
        });
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(shape::Plane::from_size(500.0).into()),
    //     material: materials.add(Color::rgba(0.3, 0.5, 0.3, 0.4).into()),
    //     ..default()
    // });

    commands.insert_resource(AmbientLight {
        color: Color::AZURE,
        brightness: 0.02,
    });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(1.0, 0.9, 0.9),
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        // cascade_shadow_config: CascadeShadowConfigBuilder {
        //     first_cascade_far_bound: 4.0,
        //     maximum_distance: 10.0,
        //     ..default()
        // }
        //     .into(),
        // ..default()
    });

    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 3000.0,
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(25.0, 5.0, 50.0),
    //     ..default()
    // });
    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(50.0, 10.0, 50.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        })
        .insert(FlyCamera::default());
}

/// Make the lights move around
///
/// Would be much cooler with day night cycle
fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() * 0.5);
    }
}
