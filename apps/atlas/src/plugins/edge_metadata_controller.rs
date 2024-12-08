use bevy::prelude::*;
use bevy_http_client::{prelude::{TypedRequest, TypedResponse}, HttpClient};

use std::fmt;

use crate::{NapkinSettings, NapkinEdgeMetadata};

pub struct EdgeMetadataControllerPlugin;

impl Plugin for EdgeMetadataControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                run_edge_metadata_controller,
            )
        );
    }
}

fn run_edge_metadata_controller(
    mut napkin: ResMut<NapkinSettings>,
    mut ev_response: EventReader<TypedResponse<Vec<NapkinEdgeMetadata>>>,
) {
    for response in ev_response.read() {
        info!("Received edges from server");
        napkin.edge_metadata = response.to_vec();
    }
}
