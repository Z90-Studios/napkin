use bevy::prelude::*;
use bevy_http_client::HttpClient;
use std::fmt;

use crate::{AtlasDiagnostics, NapkinSettings};

pub struct NapkinPlugin;

impl Plugin for NapkinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, run_napkin_controller);
    }
}

#[derive(Component)]
pub struct Napkin {}

impl Default for Napkin {
    fn default() -> Self {
        Self {}
    }
}

impl fmt::Display for Napkin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Napkin controller initialized.")
    }
}

fn run_napkin_controller(
    mut atlas_diagnostics: ResMut<AtlasDiagnostics>,
    mut napkin: ResMut<NapkinSettings>,
    mut project_request: EventWriter<
        bevy_http_client::prelude::TypedRequest<Vec<crate::NapkinProject>>,
    >,
    mut node_request: EventWriter<bevy_http_client::prelude::TypedRequest<Vec<crate::NapkinNode>>>,
    mut edge_request: EventWriter<bevy_http_client::prelude::TypedRequest<Vec<crate::NapkinEdge>>>,
    mut node_metadata_request: EventWriter<
        bevy_http_client::prelude::TypedRequest<Vec<crate::NapkinNodeMetadata>>,
    >,
    mut edge_metadata_request: EventWriter<
        bevy_http_client::prelude::TypedRequest<Vec<crate::NapkinEdgeMetadata>>,
    >,
) {
    let time = atlas_diagnostics.uptime;
    if time % 5.0 < 0.016 || time % 5.0 > 4.984 { // Adjusted to trigger around every 5 seconds, independent of frame rate
        project_request.send(
            HttpClient::new()
                .get(format!("{}/project", napkin.server_url))
                .with_type::<Vec<crate::NapkinProject>>(),
        );
        node_request.send(
            HttpClient::new()
                .get(format!("{}/node", napkin.server_url))
                .with_type::<Vec<crate::NapkinNode>>(),
        );
        edge_request.send(
            HttpClient::new()
                .get(format!("{}/edge", napkin.server_url))
                .with_type::<Vec<crate::NapkinEdge>>(),
        );
        node_metadata_request.send(
            HttpClient::new()
                .get(format!("{}/node/metadata", napkin.server_url))
                .with_type::<Vec<crate::NapkinNodeMetadata>>(),
        );
        edge_metadata_request.send(
            HttpClient::new()
                .get(format!("{}/edge/metadata", napkin.server_url))
                .with_type::<Vec<crate::NapkinEdgeMetadata>>(),
        );
    }
}
