use super::SceneCamera;
use crate::schedule::{PreStartupSet, UpdateSet};
use bevy::input::mouse::MouseMotion;
use bevy::pbr::ShadowFilteringMethod;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

#[derive(Clone, Debug)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
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

#[derive(Debug, Resource)]
struct PivotPoint(Vec3);

impl Plugin for BasicCameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PivotPoint(Vec3::ZERO))
            .add_systems(PreStartup, spawn_camera.in_set(PreStartupSet::SpawnWorld))
            .add_systems(Update, camera_control.in_set(UpdateSet::UserInputEffects));
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

fn camera_control(
    mut query: Query<&mut Transform, With<SceneCamera>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    pivot_point: Res<PivotPoint>,
) {
    if keyboard_input.pressed(KeyCode::SuperLeft) && mouse_button.pressed(MouseButton::Left) {
        let mut camera_transform = query.single_mut();

        for event in mouse_motion_events.read() {
            let rotation_speed = 0.01;
            let pivot = pivot_point.0;

            let direction = camera_transform.translation - pivot;
            let rotation = Quat::from_rotation_y(-event.delta.x * rotation_speed);
            let new_direction = rotation * direction;

            camera_transform.translation = pivot + new_direction;
            camera_transform.look_at(pivot, Vec3::Y);
        }
    }
}
