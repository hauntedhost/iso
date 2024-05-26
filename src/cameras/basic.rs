use super::SceneCamera;
use crate::schedule::{PreStartupSet, UpdateSet};
use bevy::input::mouse::MouseMotion;
use bevy::pbr::ShadowFilteringMethod;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

const PIVOT_POINT: Vec3 = Vec3::ZERO;
const ROTATION_SPEED: f32 = 0.02;

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

impl Plugin for BasicCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, spawn_camera.in_set(PreStartupSet::SpawnWorld))
            .add_systems(
                Update,
                (camera_control, snap_camera)
                    .chain()
                    .in_set(UpdateSet::UserInputEffects),
            );
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
            transform: Transform::from_xyz(5.0, 5.0, 5.0).looking_at(PIVOT_POINT, Vec3::Y),
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
) {
    if keyboard_input.pressed(KeyCode::SuperLeft) && mouse_button.pressed(MouseButton::Left) {
        let mut camera_transform = query.single_mut();

        for event in mouse_motion_events.read() {
            let direction = camera_transform.translation - PIVOT_POINT;
            let rotation = Quat::from_rotation_y(-event.delta.x * ROTATION_SPEED);
            let new_direction = rotation * direction;

            camera_transform.translation = PIVOT_POINT + new_direction;
            camera_transform.look_at(PIVOT_POINT, Vec3::Y);
        }
    }
}

fn snap_camera(
    mut query: Query<&mut Transform, With<SceneCamera>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    if mouse_button.just_released(MouseButton::Left) {
        let mut camera_transform = query.single_mut();
        let current_position = camera_transform.translation;

        let cardinal_directions = [
            Vec3::new(5.0, 5.0, 5.0),
            Vec3::new(7.0, 5.0, 0.0),
            Vec3::new(5.0, 5.0, -5.0),
            Vec3::new(0.0, 5.0, -7.0),
            Vec3::new(-5.0, 5.0, -5.0),
            Vec3::new(-7.0, 5.0, 0.0),
            Vec3::new(-5.0, 5.0, 5.0),
            Vec3::new(0.0, 5.0, 7.0),
        ];

        // Find the nearest cardinal direction
        let mut nearest_direction = cardinal_directions[0];
        let mut min_distance = current_position.distance_squared(nearest_direction);

        for &direction in &cardinal_directions[1..] {
            let distance = current_position.distance_squared(direction);
            if distance < min_distance {
                min_distance = distance;
                nearest_direction = direction;
            }
        }

        // Snap the camera to the nearest direction
        camera_transform.translation = nearest_direction;
        camera_transform.look_at(PIVOT_POINT, Vec3::Y);
    }
}
