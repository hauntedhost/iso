// use crate::{cameras::SceneCamera, schedule::UpdateSet, Terrain};
use crate::cameras::SceneCamera;
use crate::schedule::{StartupSet, UpdateSet};
use crate::terrain::Terrain;
use bevy::prelude::*;
use rand::Rng;

const PLAYER_SIZE: f32 = 0.2;

#[derive(Component, Debug)]
pub struct Player;

#[derive(Clone, Debug)]
pub struct Config;

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Clone, Debug)]
pub struct PlayerPlugin {
    pub config: Config,
}

impl Default for PlayerPlugin {
    fn default() -> Self {
        Self {
            config: Config::default(),
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player.in_set(StartupSet::SpawnEntities))
            .add_systems(
                Update,
                update_player_position.in_set(UpdateSet::UserInputEffects),
            );
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain_query: Query<(&Transform, &Terrain), With<Terrain>>,
) {
    let Ok((terrain_transform, terrain)) = terrain_query.get_single() else {
        return;
    };

    let Terrain {
        width: terrain_width,
        height: terrain_height,
        depth: terrain_depth,
    } = terrain;

    // Pin player to top of the terrain
    let terrain_top_y = terrain_transform.translation.y + (terrain_height / 2.0);
    let player_y = terrain_top_y + (PLAYER_SIZE / 2.0);

    // Randomly place the player within width/depth bounds of the terrain
    let mut rng = rand::thread_rng();
    let player_x = rng.gen_range(
        terrain_transform.translation.x - terrain_width / 2.0
            ..terrain_transform.translation.x + terrain_width / 2.0,
    );
    let player_z = rng.gen_range(
        terrain_transform.translation.z - terrain_depth / 2.0
            ..terrain_transform.translation.z + terrain_depth / 2.0,
    );

    // Player cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
            transform: Transform {
                translation: Vec3::new(player_x, player_y, player_z),
                scale: Vec3::splat(PLAYER_SIZE),
                ..default()
            },
            ..default()
        },
        Player,
    ));
}

const MOVEMENT_X_SPEED: f32 = 0.085;
const MOVEMENT_Z_SPEED: f32 = 0.125;

fn update_player_position(
    mut player_query: Query<&mut Transform, (With<Player>, Without<SceneCamera>)>,
    camera_query: Query<&Transform, (With<SceneCamera>, Without<Player>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let forward = Vec3::from(camera_transform.forward());
    let right = Vec3::from(camera_transform.right());

    for mut player_transform in player_query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let mut did_transform = false;

        if keyboard_input.pressed(KeyCode::KeyW) {
            did_transform = true;
            direction += forward * MOVEMENT_Z_SPEED;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            did_transform = true;
            direction -= forward * MOVEMENT_Z_SPEED;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            did_transform = true;
            direction -= right * MOVEMENT_X_SPEED;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            did_transform = true;
            direction += right * MOVEMENT_X_SPEED;
        }

        if direction != Vec3::ZERO {
            // direction = direction.normalize();
            player_transform.translation = Vec3::new(
                player_transform.translation.x + direction.x,
                player_transform.translation.y,
                player_transform.translation.z + direction.z,
            );
        }

        if did_transform {
            // dbg!(player_transform);
            // dbg!(forward.length(), right.length());
        }
    }
}
