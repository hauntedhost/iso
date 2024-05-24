use crate::cameras::SceneCamera;
use crate::schedule::{StartupSet, UpdateSet};
use crate::socket::client::SocketStatus;
use crate::socket::request::Request;
use crate::socket::{GameSocket, GAME_ROOM};
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
                (update_player_position, broadcast_player_update)
                    .chain()
                    .in_set(UpdateSet::UserInputEffects),
            )
            .add_event::<PlayerUpdateEvent>();
    }
}

#[derive(Event, Debug)]
pub struct PlayerUpdateEvent {
    pub new_position: Vec3,
}

impl PlayerUpdateEvent {
    pub fn new(new_position: Vec3) -> Self {
        Self { new_position }
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain_query: Query<(&Transform, &Terrain), With<Terrain>>,
    mut event_writer: EventWriter<PlayerUpdateEvent>,
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

    let player_position = Vec3::new(player_x, player_y, player_z);

    event_writer.send(PlayerUpdateEvent::new(player_position));

    // Player cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
            transform: Transform {
                translation: player_position,
                scale: Vec3::splat(PLAYER_SIZE),
                ..default()
            },
            ..default()
        },
        Player,
    ));
}

fn broadcast_player_update(
    mut event_reader: EventReader<PlayerUpdateEvent>,
    mut socket: ResMut<GameSocket>,
) {
    for &PlayerUpdateEvent { new_position } in event_reader.read() {
        let player_uuid = socket.player.uuid.clone();

        // Update player in socket
        socket.update_player_position(player_uuid.clone(), new_position);

        // Broadcast player_update
        if let Some(status) = &socket.status {
            if *status == SocketStatus::Connected {
                let request =
                    Request::new_player_update(GAME_ROOM.into(), player_uuid.clone(), new_position);
                socket
                    .handle
                    .call(request)
                    .expect("player_update request error");
            }
        }
    }
}

const MOVEMENT_X_SPEED: f32 = 0.085;
const MOVEMENT_Z_SPEED: f32 = 0.125;

fn update_player_position(
    mut player_query: Query<&mut Transform, (With<Player>, Without<SceneCamera>)>,
    camera_query: Query<&Transform, (With<SceneCamera>, Without<Player>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut event_writer: EventWriter<PlayerUpdateEvent>,
) {
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let forward = Vec3::from(camera_transform.forward());
    let right = Vec3::from(camera_transform.right());

    for mut player_position in player_query.iter_mut() {
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
            player_position.translation = Vec3::new(
                player_position.translation.x + direction.x,
                player_position.translation.y,
                player_position.translation.z + direction.z,
            );
        }

        if did_transform {
            event_writer.send(PlayerUpdateEvent::new(player_position.translation));
        }
    }
}
