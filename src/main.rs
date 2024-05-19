mod cameras;
mod lighting;
mod player;
mod schedule;
mod terrain;

use bevy::prelude::*;
use cameras::CameraPlugin;
use lighting::LightingPlugin;
use player::PlayerPlugin;
use terrain::TerrainPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LightingPlugin::default())
        .add_plugins(CameraPlugin::default())
        .add_plugins(TerrainPlugin::default())
        .add_plugins(PlayerPlugin::default())
        .run();
}
