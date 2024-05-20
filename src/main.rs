mod cameras;
mod debug;
mod lighting;
mod player;
mod schedule;
mod socket;
mod terrain;

use bevy::prelude::*;
use cameras::CameraPlugin;
use debug::DebugPlugin;
use lighting::LightingPlugin;
use player::PlayerPlugin;
use socket::SocketPlugin;
use terrain::TerrainPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DebugPlugin::default())
        .add_plugins(SocketPlugin::default())
        .add_plugins(LightingPlugin::default())
        .add_plugins(CameraPlugin::default())
        .add_plugins(TerrainPlugin::default())
        .add_plugins(PlayerPlugin::default())
        .run();
}
