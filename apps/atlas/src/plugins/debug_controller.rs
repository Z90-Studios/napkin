use bevy::prelude::*;
use bevy_rapier2d::render::{DebugRenderContext, RapierDebugRenderPlugin};
use std::fmt;

pub struct DebugControllerPlugin;

impl Plugin for DebugControllerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<DebugState>()
            .add_plugins((
                RapierDebugRenderPlugin::default()
            ))
            .add_systems(Update, (
                handle_debug_state,
            )
        );
    }
}

#[derive(Resource)]
pub struct DebugState {
    pub rapier_debug_enabled: bool,
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            rapier_debug_enabled: true,
        }
    }
}

fn handle_debug_state(
    mut debug_state: ResMut<DebugState>,
    mut rapier_debug_context: ResMut<DebugRenderContext>,
) {
    rapier_debug_context.enabled = debug_state.rapier_debug_enabled;
}
