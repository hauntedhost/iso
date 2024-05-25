pub mod basic;
pub mod viewport;

use self::basic::BasicCameraPlugin;
use self::viewport::ViewportCameraPlugin;
use bevy::prelude::*;

#[derive(Clone, Debug)]
pub struct Config {
    pub camera: Camera,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            camera: Camera::Basic(BasicConfig::default()),
        }
    }
}

#[derive(Component, Debug)]
pub struct SceneCamera;

pub type BasicConfig = basic::Config;
pub type ViewportConfig = viewport::Config;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Camera {
    Basic(BasicConfig),
    Viewport(ViewportConfig),
}

#[derive(Clone, Debug)]
pub struct CameraPlugin {
    pub config: Config,
}

impl Default for CameraPlugin {
    fn default() -> Self {
        Self {
            config: Config::default(),
        }
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        match &self.config.camera {
            Camera::Basic(config) => app.add_plugins(BasicCameraPlugin {
                config: config.clone(),
            }),
            Camera::Viewport(config) => app.add_plugins(ViewportCameraPlugin {
                config: config.clone(),
            }),
        };
    }
}
