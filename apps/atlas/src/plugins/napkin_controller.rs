use bevy::prelude::*;
use bevy_http_client::prelude::*;

use crate::NapkinSettings;
use crate::types::napkin_types::*;

pub struct NapkinPlugin;

impl Plugin for NapkinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, run_napkin_controller);
    }
}

fn run_napkin_controller(
    time: Res<Time>,
    mut napkin: ResMut<NapkinSettings>,
    mut project_request: EventWriter<TypedRequest<Vec<NapkinProject>>>,
    mut node_request: EventWriter<TypedRequest<Vec<NapkinNode>>>,
    mut edge_request: EventWriter<TypedRequest<Vec<NapkinEdge>>>,
    mut node_metadata_request: EventWriter<TypedRequest<Vec<NapkinNodeMetadata>>>,
    mut edge_metadata_request: EventWriter<TypedRequest<Vec<NapkinEdgeMetadata>>>,
) {
    napkin.uptime.tick(time.delta());
    napkin.refresh_timer.tick(time.delta());

    if napkin.refresh_timer.finished() || napkin.initialized == false {
        napkin.initialized = true;
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
