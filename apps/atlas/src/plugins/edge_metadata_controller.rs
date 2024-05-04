use bevy::prelude::*;
use bevy_http_client::prelude::TypedResponse;
use std::fmt;

use crate::{NapkinEdgeMetadata, NapkinSettings};

pub struct EdgeMetadataControllerPlugin;

impl Plugin for EdgeMetadataControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                run_edge_metadata_controller,
            ),
        );
    }
}


pub fn run_edge_metadata_controller(
  mut napkin: ResMut<NapkinSettings>,
  mut ev_response: EventReader<TypedResponse<Vec<NapkinEdgeMetadata>>>,
) {
  for response in ev_response.read() {
      info!("Received edge metadata from server");
      napkin.is_connected = true;
      napkin.edge_metadata = response.to_vec();
  }
}