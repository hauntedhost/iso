use crate::schedule::PreStartupSet;
use bevy::prelude::*;

const TERRAIN_WIDTH: f32 = 5.0;
const TERRAIN_HEIGHT: f32 = 0.3;
const TERRAIN_DEPTH: f32 = 5.0;

#[derive(Component, Debug)]
pub struct Terrain {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

#[derive(Clone, Debug)]
pub struct TerrainPlugin {}

impl Default for TerrainPlugin {
    fn default() -> Self {
        Self {}
    }
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, spawn_terrain.in_set(PreStartupSet::SpawnWorld));
    }
}

fn spawn_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(TERRAIN_WIDTH, TERRAIN_HEIGHT, TERRAIN_DEPTH)),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.3, 0.5, 0.3),
                reflectance: 0.01,
                ..default()
            }),
            // Adjust position so top surface is at y = 0
            transform: Transform::from_xyz(0.0, -TERRAIN_HEIGHT / 2.0, 0.0),
            ..default()
        },
        // Include height in tag so other queries can use this instead of TERRAIN_HEIGHT
        Terrain {
            width: TERRAIN_WIDTH,
            height: TERRAIN_HEIGHT,
            depth: TERRAIN_DEPTH,
        },
        Name::new("Terrain"),
    ));
}
