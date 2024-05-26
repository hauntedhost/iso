use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use iyes_perf_ui::diagnostics::{PerfUiEntryEntityCount, PerfUiEntryFPS};
use iyes_perf_ui::{PerfUiPlugin, PerfUiRoot};

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct DevSettings {}

#[derive(Clone, Debug)]
pub struct DevToolsPlugin {
    pub enabled: bool,
}

impl Default for DevToolsPlugin {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        if self.enabled {
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
