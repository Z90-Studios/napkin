use bevy::prelude::*;
use bevy_http_client::{prelude::{TypedRequest, TypedResponse}, HttpClient};

use std::fmt;

use crate::{NapkinSettings, NapkinNodeMetadata};

pub struct NodeMetadataControllerPlugin;

impl Plugin for NodeMetadataControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                run_node_metadata_controller,
            )
        );
    }
}

fn run_node_metadata_controller(
    mut napkin: ResMut<NapkinSettings>,
    mut ev_response: EventReader<TypedResponse<Vec<NapkinNodeMetadata>>>,
) {
    for response in ev_response.read() {
        info!("Received nodes from server");
        napkin.node_metadata = response.to_vec();
    }
}
