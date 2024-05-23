use bevy::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};
use bevy_rapier3d::render::{DebugRenderContext, RapierDebugRenderPlugin};
use std::fmt;

use crate::AtlasDiagnostics;

pub struct DebugControllerPlugin;

impl Plugin for DebugControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
          RapierDebugRenderPlugin::default(),
          InfiniteGridPlugin,
        ))
            .add_systems(Update, handle_debug_state);
    }
}

#[derive(Component)]
pub struct DebugController {
    pub visible: bool,
}

impl Default for DebugController {
    fn default() -> Self {
        Self { visible: true }
    }
}

impl fmt::Display for DebugController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Debug controller registered ( visible = {:?} )",
            self.visible
        )
    }
}

fn handle_debug_state(
  atlas_diagnostics: Res<AtlasDiagnostics>,
  mut debug_query: Query<Entity, With<DebugController>>,
  mut rapier_debug_context: ResMut<DebugRenderContext>,
  mut commands: Commands,
) {
  if atlas_diagnostics.debug_mode {
      rapier_debug_context.enabled = true;
      if debug_query.get_single().is_err() {
          commands.spawn((
              InfiniteGridBundle {
                  settings: InfiniteGridSettings {
                      x_axis_color: Color::rgb(0.8, 0.8, 0.8),
                      z_axis_color: Color::rgb(0., 1., 0.),
                      scale: 10.,
                      ..Default::default()
                  },
                  ..Default::default()
              },
              DebugController::default(),
          ));
      }
  } else {
      rapier_debug_context.enabled = false;
      for entity in debug_query.iter_mut() {
          commands.entity(entity).despawn();
      }
  }
}
