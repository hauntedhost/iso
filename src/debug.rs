use bevy::prelude::*;
use iyes_perf_ui::diagnostics::{PerfUiEntryEntityCount, PerfUiEntryFPS};
use iyes_perf_ui::{PerfUiPlugin, PerfUiRoot};

#[derive(Clone, Debug)]
pub struct Config {
    enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Debug)]
pub struct DebugPlugin {
    pub config: Config,
}

impl Default for DebugPlugin {
    fn default() -> Self {
        Self {
            config: Config::default(),
        }
    }
}

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if self.config.enabled {
            app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
                .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
                .add_plugins(PerfUiPlugin)
                .add_systems(Startup, add_perf);
        }
    }
}

fn add_perf(mut commands: Commands) {
    commands.spawn((
        PerfUiRoot { ..default() },
        PerfUiEntryFPS::default(),
        PerfUiEntryEntityCount::default(),
    ));
}
