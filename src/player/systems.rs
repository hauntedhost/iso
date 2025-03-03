use super::store::PlayerStore;
use crate::cameras::SceneCamera;
use crate::socket::client::SocketStatus;
use crate::socket::request::Request;
use crate::socket::{Socket, GAME_ROOM};
use crate::terrain::Terrain;
use bevy::prelude::*;
use rand::Rng;
use std::time::Duration;

pub const PLAYER_SIZE: f32 = 0.2;
const BROADCAST_THROTTLE_MS: u64 = 30;

// TODO: This is weird
const MOVEMENT_X_SPEED: f32 = 0.085;
const MOVEMENT_Z_SPEED: f32 = 0.125;

#[derive(Component, Debug)]
pub struct PlayerTag;

#[derive(Component, Debug)]
pub struct FriendTag {
    pub player_uuid: String,
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

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
pub struct BroadcastBuffer {
    pub last_update: Option<Vec3>,
    pub timer: Timer,
}

impl Default for BroadcastBuffer {
    fn default() -> Self {
        Self {
            last_update: None,
            timer: Timer::new(
                Duration::from_millis(BROADCAST_THROTTLE_MS),
                TimerMode::Repeating,
            ),
        }
    }
}

#[derive(Event, Debug)]
pub struct FriendUpdateEvent {
    pub player_uuid: String,
    pub new_position: Vec3,
}

impl FriendUpdateEvent {
    pub fn new(player_uuid: String, new_position: Vec3) -> Self {
        Self {
            player_uuid,
            new_position,
        }
    }
}

pub fn spawn_player(
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
        PlayerTag,
        Name::new("Player"),
    ));
}

pub fn update_player_position(
    mut player_query: Query<
        &mut Transform,
        (With<PlayerTag>, Without<FriendTag>, Without<SceneCamera>),
    >,
    camera_query: Query<&Transform, (With<SceneCamera>, Without<PlayerTag>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut event_writer: EventWriter<PlayerUpdateEvent>,
) {
    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let forward = Vec3::from(camera_transform.forward());
    let right = Vec3::from(camera_transform.right());

    // TODO: there should really just be one player
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

pub fn broadcast_player_update(
    mut event_reader: EventReader<PlayerUpdateEvent>,
    mut store: ResMut<PlayerStore>,
    mut broadcast_buffer: ResMut<BroadcastBuffer>,
    socket: Res<Socket>,
    time: Res<Time>,
) {
    broadcast_buffer.timer.tick(time.delta());

    let player_uuid = store.player_uuid.clone();
    for &PlayerUpdateEvent { new_position } in event_reader.read() {
        // Replace last_update in buffer
        broadcast_buffer.last_update = Some(new_position);

        // Update player in socket
        store.update_player_position(player_uuid.clone(), new_position);
    }

    if broadcast_buffer.timer.finished() {
        if let Some(new_position) = broadcast_buffer.last_update.take() {
            if let Some(status) = &socket.status {
                if *status == SocketStatus::Connected {
                    let request = Request::new_player_update(
                        GAME_ROOM.into(),
                        player_uuid.clone(),
                        new_position,
                    );
                    socket
                        .handle
                        .call(request)
                        .expect("player_update request error");
                }
            }
        }
        broadcast_buffer.timer.reset();
    }
}

pub fn spawn_friends(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut update_event_reader: EventReader<FriendUpdateEvent>,
    mut store: ResMut<PlayerStore>,
) {
    for FriendUpdateEvent {
        player_uuid,
        new_position,
    } in update_event_reader.read()
    {
        // Skip if player is self
        if player_uuid.clone() == store.player_uuid {
            continue;
        }

        // Skip if player already spawned
        if store.has_spawned(player_uuid) {
            continue;
        }

        // Spawn new player
        store.set_spawned_at(player_uuid);

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::default()),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6)),
                transform: Transform {
                    translation: new_position.clone(),
                    scale: Vec3::splat(PLAYER_SIZE),
                    ..default()
                },
                ..default()
            },
            PlayerTag,
            FriendTag {
                player_uuid: player_uuid.clone(),
            },
            Name::new("Friend"),
        ));
    }
}

pub fn update_friend_positions(
    mut friend_query: Query<(&mut Transform, &FriendTag), With<FriendTag>>,
    store: Res<PlayerStore>,
) {
    for (mut current_position, friend_tag) in friend_query.iter_mut() {
        // Skip unless player can be found in friends
        let Some(player) = store.get_friend(&friend_tag.player_uuid) else {
            continue;
        };

        // Skip unless friend position exists
        let Some(new_position) = player.position else {
            continue;
        };

        // Skip if no position change
        if current_position.translation == new_position {
            continue;
        }

        current_position.translation = new_position;
    }
}

// Despawn friend entities that cannot be found in socket.friends
pub fn despawn_friends(
    mut commands: Commands,
    friend_query: Query<(Entity, &FriendTag), With<FriendTag>>,
    store: Res<PlayerStore>,
) {
    for (entity, friend_tag) in friend_query.iter() {
        if store.get_friend(&friend_tag.player_uuid).is_none() {
            commands.entity(entity).despawn_recursive();
        };
    }
}
