use bevy::prelude::*;
use bevy_http_client::prelude::TypedResponse;
use std::fmt;

use crate::{NapkinProject, NapkinSettings};

pub struct ProjectControllerPlugin;

impl Plugin for ProjectControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                run_project_controller,
            ),
        );
    }
}


pub fn run_project_controller(
  mut napkin: ResMut<NapkinSettings>,
  mut ev_response: EventReader<TypedResponse<Vec<NapkinProject>>>,
) {
  for response in ev_response.read() {
      info!("Received projects list from server");
      napkin.is_connected = true;
      napkin.projects = response.to_vec();
  }
}