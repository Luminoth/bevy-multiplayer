#![cfg(not(feature = "server"))]

use bevy::{
    diagnostic::{
        DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
        SystemInformationDiagnosticsPlugin,
    },
    input::common_conditions::input_toggle_active,
    prelude::*,
};
use bevy_egui::{egui, EguiContexts};

use crate::AppState;

#[derive(Debug, Default, Reflect, Resource)]
pub struct DebugSettings {
    pub show_world_inspector: bool,
    pub show_state_inspector: bool,
}

fn show_world_inspector(debug_settings: Res<DebugSettings>) -> bool {
    debug_settings.show_world_inspector
}

fn show_state_inspector(debug_settings: Res<DebugSettings>) -> bool {
    debug_settings.show_state_inspector
}

#[derive(Debug, Default)]
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // diagnostics
        app.add_plugins((
            //bevy::diagnostic::LogDiagnosticsPlugin::default(),
            bevy::diagnostic::FrameTimeDiagnosticsPlugin,
            bevy::diagnostic::EntityCountDiagnosticsPlugin,
            //bevy::render::diagnostic::RenderDiagnosticsPlugin,
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
        ));

        // inspectors
        app.add_plugins((
            // TODO: why does the world inspector not pick up custom resource types?
            // using register_type() on them doesn't seem to fix it
            // TODO: might have outgrown the quick plugins: https://docs.rs/bevy-inspector-egui/0.25.2/bevy_inspector_egui/#use-case-2-manual-ui
            bevy_inspector_egui::quick::WorldInspectorPlugin::default()
                .run_if(show_world_inspector),
            bevy_inspector_egui::quick::StateInspectorPlugin::<AppState>::default()
                .run_if(show_state_inspector),
        ));

        app.init_resource::<DebugSettings>();

        app.add_systems(
            Update,
            // TODO: this needs to be reworked to also hide the inspectors when disabling
            // (probably just copy input_toggle_active but also have it disable everything?)
            debug_ui.run_if(input_toggle_active(false, KeyCode::Backquote)),
        );
    }
}

fn debug_ui(
    time: Res<Time>,
    diagnostics: Res<DiagnosticsStore>,
    mut debug_settings: ResMut<DebugSettings>,
    mut contexts: EguiContexts,
) {
    egui::Window::new("Debug").show(contexts.ctx_mut(), |ui| {
        ui.vertical(|ui| {
            ui.label(format!(
                "{:.1} avg fps, {:.3} avg ms/frame",
                diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FPS)
                    .and_then(|fps| fps.smoothed())
                    .unwrap_or_default(),
                diagnostics
                    .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
                    .and_then(|frame_time| frame_time.smoothed())
                    .unwrap_or_else(|| time.delta_seconds_f64())
            ));
            ui.label(format!(
                "{:.2}% avg cpu, {:.2}% memory",
                diagnostics
                    .get(&SystemInformationDiagnosticsPlugin::CPU_USAGE)
                    .and_then(|cpu| cpu.smoothed())
                    .unwrap_or_default(),
                diagnostics
                    .get(&SystemInformationDiagnosticsPlugin::MEM_USAGE)
                    .and_then(|memory| memory.value())
                    .unwrap_or_default()
            ));
            ui.label(format!(
                "{} entities",
                diagnostics
                    .get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
                    .and_then(|count| count.value())
                    .unwrap_or_default()
            ));

            if ui.button("World Inspector").clicked() {
                debug_settings.show_world_inspector = !debug_settings.show_world_inspector;
            }

            if ui.button("State Inspector").clicked() {
                debug_settings.show_state_inspector = !debug_settings.show_state_inspector;
            }
        });
    });
}
