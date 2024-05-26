mod cameras;
mod collision;
mod debug;
mod helpers;
mod lighting;
mod player;
mod schedule;
mod socket;
mod terrain;

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use cameras::CameraPlugin;
use collision::CollisionPlugin;
use debug::DebugPlugin;
use helpers::names::get_from_env_or_generate_window_title;
use lighting::LightingPlugin;
use player::PlayerPlugin;
use socket::SocketPlugin;
use terrain::TerrainPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: get_from_env_or_generate_window_title(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(DebugPlugin::default())
        .add_plugins(SocketPlugin::default())
        .add_plugins(LightingPlugin::default())
        .add_plugins(CameraPlugin::default())
        .add_plugins(TerrainPlugin::default())
        .add_plugins(PlayerPlugin::default())
        .add_plugins(CollisionPlugin::default())
        .run();
}
