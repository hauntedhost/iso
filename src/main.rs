mod cameras;
mod collision;
mod dev_tools;
mod helpers;
mod lighting;
mod player;
mod schedule;
mod socket;
mod terrain;

use bevy::prelude::*;
use cameras::CameraPlugin;
use collision::CollisionPlugin;
use dev_tools::DevToolsPlugin;
use helpers::names::get_title_from_env_or_generate;
use lighting::LightingPlugin;
use player::PlayerPlugin;
use socket::SocketPlugin;
use terrain::TerrainPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: get_title_from_env_or_generate(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(DevToolsPlugin::default())
        // TODO: GameStatePlugin for player_uuid and players
        .add_plugins(SocketPlugin::default())
        .add_plugins(LightingPlugin::default())
        .add_plugins(CameraPlugin::default())
        .add_plugins(TerrainPlugin::default())
        .add_plugins(PlayerPlugin::default())
        .add_plugins(CollisionPlugin::default())
        .run();
}
