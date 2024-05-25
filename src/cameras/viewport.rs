use super::SceneCamera;
use crate::schedule::{PreStartupSet, UpdateSet};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct Config {
    pub title: String,
    pub resolution: (f32, f32),
}

impl Default for Config {
    fn default() -> Self {
        let default = Window::default();
        let width = default.resolution.physical_width() as f32;
        let height = default.resolution.physical_height() as f32;

        Self {
            title: "Untitled".to_string(),
            resolution: (width, height),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ViewportCameraPlugin {
    pub config: Config,
}

impl Default for ViewportCameraPlugin {
    fn default() -> Self {
        Self {
            config: Config::default(),
        }
    }
}

impl Plugin for ViewportCameraPlugin {
    fn build(&self, app: &mut App) {
        let config = self.config.clone();

        app.add_systems(
            PreStartup,
            (move |query: Query<&mut Window>, commands: Commands| {
                spawn_camera(query, commands, &config);
            })
            .in_set(PreStartupSet::SpawnWorld),
        )
        .add_systems(Update, pan_orbit_camera.in_set(UpdateSet::UserInputEffects));
    }
}

#[derive(Component)]
struct ViewportCamera {
    focus: Vec3,
    radius: f32,
    upside_down: bool,
}

impl Default for ViewportCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

fn spawn_camera(mut query: Query<&mut Window>, mut commands: Commands, config: &Config) {
    let Ok(mut window) = query.get_single_mut() else {
        return;
    };

    let (width, height) = config.resolution;
    window.title = config.title.clone();
    window.resolution.set(width, height);

    let translation = Vec3::new(10.0, 20.0, 50.0);
    let radius = translation.length();

    let transform = Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y);

    let camera = Camera3dBundle {
        transform,
        ..default()
    };

    let pan_orbit = ViewportCamera {
        radius,
        ..default()
    };

    commands.spawn((camera, pan_orbit, SceneCamera));
}

fn pan_orbit_camera(
    windows: Query<&Window>,
    mut query: Query<(&mut ViewportCamera, &mut Transform, &Projection)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_scroll: EventReader<MouseWheel>,
) {
    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    // Orbit
    if keyboard_input.pressed(KeyCode::SuperLeft) && mouse_input.pressed(MouseButton::Left) {
        for ev in mouse_motion.read() {
            rotation_move += ev.delta;
        }
    // Pan
    } else if keyboard_input.pressed(KeyCode::Space) && mouse_input.pressed(MouseButton::Left) {
        for ev in mouse_motion.read() {
            pan += ev.delta;
        }
    // Zoom
    } else if keyboard_input.pressed(KeyCode::ShiftLeft) {
        for ev in mouse_scroll.read() {
            scroll += ev.y;
        }
    }

    if keyboard_input.just_released(KeyCode::SuperLeft)
        || keyboard_input.just_pressed(KeyCode::SuperLeft)
    {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this frame
            // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let window = get_primary_window_size(&windows);
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            any = true;
            // make panning distance independent of resolution and FOV,
            let window = get_primary_window_size(&windows);
            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(projection.fov * projection.aspect_ratio, projection.fov) / window;
            }
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            any = true;
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation =
                pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }

    // consume any remaining events, so they don't pile up if we don't need them
    // (and also to avoid Bevy warning us about not checking events every frame update)
    mouse_motion.clear();
}

fn get_primary_window_size(windows: &Query<&Window>) -> Vec2 {
    let window = windows.single();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}
