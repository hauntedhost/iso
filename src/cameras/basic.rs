use super::SceneCamera;
use crate::schedule::{PreStartupSet, UpdateSet};
use bevy::pbr::ShadowFilteringMethod;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Config {
    pub rotation: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { rotation: false }
    }
}

#[derive(Clone, Debug)]
pub struct BasicCameraPlugin {
    pub config: Config,
}

impl Default for BasicCameraPlugin {
    fn default() -> Self {
        Self {
            config: Config::default(),
        }
    }
}

impl Plugin for BasicCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, spawn_camera.in_set(PreStartupSet::SpawnWorld));

        if self.config.rotation {
            app.add_systems(
                Update,
                camera_movement
                    .in_set(UpdateSet::UserInputEffects)
                    .run_if(on_timer(Duration::from_millis(10))),
            );
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            projection: OrthographicProjection {
                // 6 world units per window height
                scaling_mode: ScalingMode::FixedVertical(6.0),
                ..default()
            }
            .into(),
            transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        SceneCamera,
        ShadowFilteringMethod::Castano13,
    ));
}

fn camera_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<SceneCamera>>,
) {
    for mut transform in query.iter_mut() {
        let mut did_transform = false;

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            did_transform = true;
            transform.translation.y += 0.25;
        }

        if keyboard_input.pressed(KeyCode::ArrowDown) {
            did_transform = true;
            transform.translation.y -= 0.25;
        }

        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            did_transform = true;
            transform.translation.x -= 0.25;
        }

        if keyboard_input.pressed(KeyCode::ArrowRight) {
            did_transform = true;
            transform.translation.x += 0.25;
        }

        if did_transform {
            transform.look_at(Vec3::ZERO, Vec3::Y);
            // dbg!(transform);
        }
    }
}
