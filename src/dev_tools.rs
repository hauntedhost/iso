use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
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

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct DevSettings {}

#[derive(Clone, Debug)]
pub struct DevToolsPlugin {
    pub config: Config,
}

impl Default for DevToolsPlugin {
    fn default() -> Self {
        Self {
            config: Config::default(),
        }
    }
}

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        if self.config.enabled {
            app.insert_resource(DevSettings::default())
                .register_type::<DevSettings>()
                .add_plugins(WorldInspectorPlugin::new())
                .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
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
        Name::new("PerfUi"),
    ));
}
