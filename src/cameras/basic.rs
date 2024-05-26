use super::SceneCamera;
use crate::schedule::{PreStartupSet, UpdateSet};
use bevy::input::mouse::MouseMotion;
use bevy::pbr::ShadowFilteringMethod;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

const ANIMATION_SPEED: f32 = 15.0;
const PIVOT_POINT: Vec3 = Vec3::ZERO;
const USER_ROTATION_SPEED: f32 = 0.02;

#[derive(Clone, Debug)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Component)]
struct CameraAnimation {
    target_position: Vec3,
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
                (handle_user_rotation, insert_snap_animation)
                    .chain()
                    .in_set(UpdateSet::UserInputEffects),
            )
            .add_systems(Update, handle_animation.in_set(UpdateSet::AfterEffects));
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

fn handle_user_rotation(
    mut query: Query<&mut Transform, With<SceneCamera>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if keyboard_input.pressed(KeyCode::SuperLeft) && mouse_button.pressed(MouseButton::Left) {
        let mut camera_transform = query.single_mut();

        for event in mouse_motion_events.read() {
            let direction = camera_transform.translation - PIVOT_POINT;
            let rotation = Quat::from_rotation_y(-event.delta.x * USER_ROTATION_SPEED);
            let new_direction = rotation * direction;

            camera_transform.translation = PIVOT_POINT + new_direction;
            camera_transform.look_at(PIVOT_POINT, Vec3::Y);
        }
    }
}

fn insert_snap_animation(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform), With<SceneCamera>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
) {
    if mouse_button.just_released(MouseButton::Left) {
        let (entity, camera_transform) = query.single_mut();
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

        // Initiate camera animation to snap to the nearest cardinal direction
        commands.entity(entity).insert(CameraAnimation {
            target_position: nearest_direction,
        });
    }
}

fn handle_animation(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &CameraAnimation), With<SceneCamera>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
) {
    for (entity, mut transform, animation) in query.iter_mut() {
        if mouse_button.pressed(MouseButton::Left) {
            commands.entity(entity).remove::<CameraAnimation>();
            continue;
        }

        let direction = animation.target_position - transform.translation;
        let distance = direction.length();
        let normalized_direction = if distance > f32::EPSILON {
            direction.normalize()
        } else {
            Vec3::ZERO
        };

        let movement = normalized_direction * ANIMATION_SPEED * time.delta_seconds();

        if movement.length() >= distance {
            transform.translation = animation.target_position;
            commands.entity(entity).remove::<CameraAnimation>();
        } else {
            transform.translation += movement;
        }

        transform.look_at(PIVOT_POINT, Vec3::Y);
    }
}
