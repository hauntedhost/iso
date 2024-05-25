use crate::cameras::SceneCamera;
use crate::player::{FriendTag, PlayerTag};
use crate::schedule::{PreStartupSet, UpdateSet};
use bevy::prelude::*;

const SCENE_LIGHT_POS: Vec3 = Vec3::new(-2.0, 1.4, 0.5);
const SCENE_LIGHT_TARGET_POS: Vec3 = Vec3::new(-0.02, 0.02, 0.02);
const SPOTLIGHT_POS: Vec3 = Vec3::new(0.0, 1.0, 0.0);
const MOVEMENT_SPEED: f32 = 0.01;

#[derive(Component, Debug)]
pub struct PlayerLight;

#[derive(Component, Debug)]
pub struct SceneLight;

#[derive(Component, Debug)]
pub struct SceneLightTarget;

#[derive(Clone, Debug)]
pub struct Config {
    pub scene_light_controls: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scene_light_controls: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LightingPlugin {
    pub config: Config,
}

impl Default for LightingPlugin {
    fn default() -> Self {
        Self {
            config: Config::default(),
        }
    }
}

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, spawn_lights.in_set(PreStartupSet::SpawnWorld))
            .add_systems(
                Update,
                follow_player_with_spotlight.in_set(UpdateSet::AfterEffects),
            );

        if self.config.scene_light_controls {
            app.add_systems(
                PreStartup,
                spawn_scene_light_target.in_set(PreStartupSet::SpawnWorld),
            )
            .add_systems(
                Update,
                scene_light_movement.in_set(UpdateSet::UserInputEffects),
            );
        }
    }
}

fn spawn_lights(mut commands: Commands) {
    // Scene light
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 2_000.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_translation(SCENE_LIGHT_POS)
                .looking_at(SCENE_LIGHT_TARGET_POS, Vec3::Y),
            ..default()
        },
        SceneLight,
    ));

    // Player tracking spotlight
    commands.spawn((
        SpotLightBundle {
            spot_light: SpotLight {
                color: Color::ANTIQUE_WHITE,
                shadows_enabled: true,
                intensity: 20_000.0,
                ..default()
            },
            transform: Transform::from_translation(SPOTLIGHT_POS).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PlayerLight,
    ));
}

fn follow_player_with_spotlight(
    mut player_light_query: Query<&mut Transform, (With<PlayerLight>, Without<PlayerTag>)>,
    player_query: Query<
        &mut Transform,
        (With<PlayerTag>, Without<FriendTag>, Without<PlayerLight>),
    >,
) {
    let (Ok(mut player_light_transform), Ok(player_transform)) = (
        player_light_query.get_single_mut(),
        player_query.get_single(),
    ) else {
        return;
    };

    let fixed_distance = 0.85;
    let direction_to_player = player_transform.translation - player_light_transform.translation;
    let new_position = Vec3::new(
        player_transform.translation.x - direction_to_player.normalize().x * fixed_distance,
        player_light_transform.translation.y,
        player_transform.translation.z - direction_to_player.normalize().z * fixed_distance,
    );

    player_light_transform.translation = new_position;
    player_light_transform.look_at(player_transform.translation, Vec3::Y);
}

fn spawn_scene_light_target(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::rgba(1.0, 0.0, 0.0, 0.5)),
            transform: Transform {
                translation: SCENE_LIGHT_TARGET_POS,
                scale: Vec3::splat(0.25),
                ..default()
            },
            ..default()
        },
        SceneLightTarget,
    ));
}

fn scene_light_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    camera_query: Query<
        &Transform,
        (
            With<SceneCamera>,
            Without<SceneLight>,
            Without<SceneLightTarget>,
        ),
    >,
    mut scene_light_query: Query<
        &mut Transform,
        (
            With<SceneLight>,
            Without<SceneLightTarget>,
            Without<SceneCamera>,
        ),
    >,
    mut scene_light_target_query: Query<
        &mut Transform,
        (
            With<SceneLightTarget>,
            Without<SceneLight>,
            Without<SceneCamera>,
        ),
    >,
) {
    let (Ok(camera_transform), Ok(mut scene_light_transform), Ok(mut scene_light_target_transform)) = (
        camera_query.get_single(),
        scene_light_query.get_single_mut(),
        scene_light_target_query.get_single_mut(),
    ) else {
        return;
    };

    let mut did_transform = false;

    // Move the light target
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        let mut direction = Vec3::ZERO;
        let forward = Vec3::from(camera_transform.forward());
        let right = Vec3::from(camera_transform.right());

        if keyboard_input.pressed(KeyCode::AltLeft) {
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                did_transform = true;
                scene_light_target_transform.translation.y -= MOVEMENT_SPEED;
            }

            if keyboard_input.pressed(KeyCode::ArrowDown) {
                did_transform = true;
                scene_light_target_transform.translation.y += MOVEMENT_SPEED;
            }
        } else {
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                // scene_light_target_transform.translation.y += MOVEMENT_SPEED;
                did_transform = true;
                direction += forward * MOVEMENT_SPEED;
            }

            if keyboard_input.pressed(KeyCode::ArrowDown) {
                // scene_light_target_transform.translation.y -= MOVEMENT_SPEED;
                did_transform = true;
                direction -= forward * MOVEMENT_SPEED;
            }

            if keyboard_input.pressed(KeyCode::ArrowLeft) {
                // scene_light_target_transform.translation.x -= MOVEMENT_SPEED;
                did_transform = true;
                direction -= right * MOVEMENT_SPEED;
            }

            if keyboard_input.pressed(KeyCode::ArrowRight) {
                // scene_light_target_transform.translation.x += MOVEMENT_SPEED;
                did_transform = true;
                direction += right * MOVEMENT_SPEED;
            }
        }

        scene_light_target_transform.translation = Vec3::new(
            scene_light_target_transform.translation.x + direction.x,
            scene_light_target_transform.translation.y,
            scene_light_target_transform.translation.z + direction.z,
        );
    }

    // Orbit around current target
    if keyboard_input.pressed(KeyCode::SuperLeft) {
        if keyboard_input.pressed(KeyCode::AltLeft) {
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                did_transform = true;
                scene_light_transform.translation.y -= MOVEMENT_SPEED;
            }

            if keyboard_input.pressed(KeyCode::ArrowDown) {
                did_transform = true;
                scene_light_transform.translation.y += MOVEMENT_SPEED;
            }
        } else {
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                did_transform = true;
                scene_light_transform.translation.z += MOVEMENT_SPEED;
            }

            if keyboard_input.pressed(KeyCode::ArrowDown) {
                did_transform = true;
                scene_light_transform.translation.z -= MOVEMENT_SPEED;
            }

            if keyboard_input.pressed(KeyCode::ArrowLeft) {
                did_transform = true;
                scene_light_transform.translation.x -= MOVEMENT_SPEED;
            }

            if keyboard_input.pressed(KeyCode::ArrowRight) {
                did_transform = true;
                scene_light_transform.translation.x += MOVEMENT_SPEED;
            }
        }
    }

    if did_transform {
        dbg!(
            scene_light_transform.translation,
            scene_light_target_transform.translation
        );
    }

    scene_light_transform.look_at(scene_light_target_transform.translation, Vec3::Y);
}
