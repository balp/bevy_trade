use std::f32::consts::PI;

use bevy::{pbr::CascadeShadowConfigBuilder, pbr::NotShadowCaster, prelude::*};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use rand::Rng;

/// Holder for map positions
///
/// the pos values to world map looks like:
/// z -- latitude (positive north, negative south)
/// y -- height
/// x -- longitude (positive east, negative west)
#[derive(Component, Debug)]
struct MapPosition {
    pos: Vec3,
}


impl MapPosition {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: Vec3::new(x, y, z),
        }
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
        .add_system(water_simulation)
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
                println!(
                    "diamond step: {:?},{:?} {:?} {:?}",
                    x, z, step_size, half_step
                );

                let top_left = height_map[x][z];
                let top_right = height_map[x + step_index][z];
                let bottom_left = height_map[x][z + step_index];
                let bottom_right = height_map[x + step_index][z + step_index];
                let middle = (top_left + top_right + bottom_right + bottom_left) / 4.0
                    + scaling(scale, step_size);
                height_map[x + half_step_index][z + half_step_index] = middle;
            }
        }
        // Square Step
        for x in (0..WIDTH - 1).step_by(step_size) {
            for z in (0..WIDTH - 1).step_by(step_size) {
                println!(
                    "square step: {:?},{:?} {:?} {:?}",
                    x, z, step_size, half_step
                );
                let top_left = height_map[x][z];
                let top_right = height_map[x + step_index][z];
                let bottom_left = height_map[x][z + step_index];
                let bottom_right = height_map[x + step_index][z + step_index];
                let middle = height_map[x + half_step][z + half_step];

                height_map[x][z + half_step_index] =
                    (top_left + middle + bottom_left) / 3.0 + scaling(scale, step_size);
                height_map[x + half_step_index][z] =
                    (top_left + middle + top_right) / 3.0 + scaling(scale, step_size);
                height_map[x + step_index][z + step_index] =
                    (top_right + middle + bottom_right) / 3.0 + scaling(scale, step_size);
                height_map[x + half_step_index][z + step_index] =
                    (bottom_left + middle + bottom_right) / 3.0 + scaling(scale, step_size);
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
                mesh: meshes.add(Mesh::from(shape::Box::new(
                    1.0,
                    height_map[x][z] + f32::abs(min),
                    1.0,
                ))),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(x as f32, min, z as f32),
                ..default()
            });
        }
    }
    commands
        .spawn(MapPosition::new(64.0, height_map[64][64], 64.0))
        .insert(WaterLevel::new(0.1))
        .insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 0.1, 1.0))),
            material: materials.add(Color::rgba(0.1, 0.01, 0.5, 0.5).into()),
            transform: Transform::from_xyz(64.0, height_map[64][64], 64.0),
            ..default()
        })
        .insert(NotShadowCaster);
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
        ..default() // The default cascade config is designed to handle large scenes.
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
            transform: Transform::from_xyz(50.0, 10.0, 50.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
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
        transform.rotate_y(time.delta_seconds() * 0.01);
    }
}


/// Simulate water flow
///
fn water_simulation(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    mut water: Query<(Entity, &MapPosition, &mut WaterLevel, &Handle<Mesh>)>,
    mut dry: Query<(Entity, &MapPosition), Without<WaterLevel>>,
) {
    let mut total_height_map: [[f32; 128]; 128] = [[0.0; 128]; 128];
    let mut water_levels: [[f32; 128]; 128] = [[0.0; 128]; 128];
    for (entity, map_pos) in dry.iter() {
        total_height_map[map_pos.pos.x as usize][map_pos.pos.z as usize] = map_pos.pos.y;
    }

    // 0. Update water source with more water (e.g. add some water to 64,64.
    for (entity, map_pos, mut level, mut mesh) in water.iter_mut() {
        if map_pos.pos.x == 64.0 && map_pos.pos.z == 64.0 {
           //  eprintln!("Add water at {:?} have water {:?}  mesh {:?}", map_pos, level, mesh);
            level.depth = level.depth + 0.1 * time.delta_seconds();
        }
        total_height_map[map_pos.pos.x as usize][map_pos.pos.z as usize] = map_pos.pos.y + level.depth;
        water_levels[map_pos.pos.x as usize][map_pos.pos.z as usize] = level.depth;
    }


    // 1. For each positions with water:
    //    Spill over water to any neighbour with lower total water level, add water level and
    //    Pbr if needed.
    for (entity, map_pos, mut level, mut mesh) in water.iter_mut() {
        // eprintln!("Entity {:?} at {:?} have water {:?}  mesh {:?}", entity, map_pos, level, mesh);
        // For north, east, south, west
        let mut my_total = map_pos.pos.y + level.depth;
        let x = map_pos.pos.x as usize;
        let z = map_pos.pos.z as usize;

        if z >= 1 {
            let other_height = total_height_map[x][z - 1];
            let level_diff = my_total -  other_height;
            //eprintln!("north me {} other {} diff {}", my_total, other_height, level_diff);
            if level_diff > 0.0 {
                let change = (level_diff / 4.0) * time.delta_seconds();
                water_levels[x][z - 1] = water_levels[x][z - 1] + change;
                water_levels[x][z] = water_levels[x][z] - change;
            }
        }
        if x <= 127 {
            let other_height = total_height_map[x+1][z];
            let level_diff = my_total -  other_height;
            //eprintln!("east me {} other {} diff {}", my_total, other_height, level_diff);
            if level_diff > 0.0 {
                let change = (level_diff / 4.0) * time.delta_seconds();
                water_levels[x+1][z] = water_levels[x+1][z] + change;
                water_levels[x][z] = water_levels[x][z] - change;
            }
        }
        if z <= 127 {
            let other_height = total_height_map[x][z+1];
            let level_diff = my_total -  other_height;
            // eprintln!("east me {} other {} diff {}", my_total, other_height, level_diff);
            if level_diff > 0.0 {
                let change = (level_diff / 4.0) * time.delta_seconds();
                water_levels[x][z+1] = water_levels[x][z+1] + change;
                water_levels[x][z] = water_levels[x][z] - change;
            }
        }
        if x >= 1 {
            let other_height = total_height_map[x-1][z];
            let level_diff = my_total -  other_height;
            // eprintln!("east me {} other {} diff {}", my_total, other_height, level_diff);
            if level_diff > 0.0 {
                let change = (level_diff / 4.0) * time.delta_seconds();
                water_levels[x-1][z] = water_levels[x-1][z] + change;
                water_levels[x][z] = water_levels[x][z] - change;
            }
        }
    }

    // println!("water: {:?}", water_levels);
    // 2. For all positions, update the water mesh if water level is new
    for (entity, map_pos, mut level, mut mesh) in water.iter_mut() {
        if level.depth != water_levels[map_pos.pos.x as usize][map_pos.pos.z as usize] {
            level.depth = water_levels[map_pos.pos.x as usize][map_pos.pos.z as usize];
            meshes.set_untracked(mesh,
                Mesh::from(shape::Box::new(1.0, level.depth, 1.0))
            );
        }
        water_levels[map_pos.pos.x as usize][map_pos.pos.z as usize] = -1.0;
    }
    for (entity, map_pos) in dry.iter() {
        if water_levels[map_pos.pos.x as usize][map_pos.pos.z as usize] > 0.0 {
            commands.entity(entity)
                .insert(
                    PbrBundle {
                        mesh: meshes.add(
                            Mesh::from(shape::Box::new(
                                1.0,
                                water_levels[map_pos.pos.x as usize][map_pos.pos.z as usize],
                                1.0))),
                        material: materials.add(Color::rgba(0.1, 0.01, 0.5, 0.5).into()),
                        transform: Transform::from_xyz(map_pos.pos.x, map_pos.pos.y, map_pos.pos.z),
                        ..default()
                    }
                )
                .insert(WaterLevel::new(water_levels[map_pos.pos.x as usize][map_pos.pos.z as usize]))
                .insert(NotShadowCaster);
        }
    }
}
