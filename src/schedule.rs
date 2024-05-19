use bevy::prelude::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum PreStartupSet {
    SpawnWorld,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum StartupSet {
    SpawnEntities,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum UpdateSet {
    UserInputEffects,
    AfterEffects,
}

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(PreStartup, PreStartupSet::SpawnWorld)
            .configure_sets(Startup, StartupSet::SpawnEntities)
            .configure_sets(
                Update,
                (UpdateSet::UserInputEffects, UpdateSet::AfterEffects).chain(),
            );
    }
}
