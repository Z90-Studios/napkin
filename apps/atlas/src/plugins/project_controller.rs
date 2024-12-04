use bevy::prelude::*;
use bevy_http_client::{prelude::{TypedRequest, TypedResponse}, HttpClient};

use crate::{NapkinEdits, NapkinProject, NapkinSettings};

pub struct ProjectControllerPlugin;

impl Plugin for ProjectControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                run_project_controller,
                save_project,
                apply_response,
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
      napkin.projects = response.to_vec();
  }
}

pub fn save_project(
    mut napkin: ResMut<NapkinSettings>,
    mut edits: ResMut<NapkinEdits>,
    mut project_request: EventWriter<TypedRequest<NapkinProject>>,
) {
    if edits.save_project {
        let mut http_client = HttpClient::new();
        if edits.project.id.is_empty() {
            http_client = http_client.post(format!("{}/project", napkin.server_url));
        } else {
            http_client = http_client.put(format!("{}/project/{}", napkin.server_url, edits.project.id));
        }
        http_client = http_client.json(&edits.project);
        project_request.send(
            http_client.with_type::<NapkinProject>(),
        );
        edits.save_project = false;
    }
}

pub fn apply_response(
    mut napkin: ResMut<NapkinSettings>,
    mut edits: ResMut<NapkinEdits>,
    mut project_response: EventReader<TypedResponse<NapkinProject>>,
) {
    for response in project_response.read() {
        info!("Received updated project from server");
        let project = NapkinProject {
            id: response.id.clone(),
            scope: response.scope.clone(),
            name: response.name.clone(),
        };
        edits.project = project.clone();
        for p in napkin.projects.iter_mut() {
            if p.id == project.id {
                *p = project.clone();
            }
        }
    }
}
