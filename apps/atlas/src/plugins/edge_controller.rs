use bevy::{
    prelude::*, sprite::Mesh2dHandle, window::{CursorGrabMode, PrimaryWindow}
};
use bevy_egui::{
    egui::{self, Color32, CursorIcon},
    EguiContexts,
};
use bevy_http_client::prelude::TypedResponse;
use bevy_rapier2d::{
    dynamics::{
        GravityScale, ImpulseJoint, RapierRigidBodyHandle, RigidBody, RopeJointBuilder
    },
    geometry::{Collider, CollisionGroups, Group, SolverGroups},
    pipeline::QueryFilter,
    plugin::RapierContext,
};
use std::{f32::consts::PI, fmt};

use crate::{NapkinEdge, NapkinSettings};

use super::node_controller::NodeController;

pub struct EdgeControllerPlugin;

impl Plugin for EdgeControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
                run_edge_controller,
                edge_spawner,
                edge_destroyer,
                handle_edge_physics,
        ));
    }
}

#[derive(Component)]
pub struct HoveredEdge;

#[derive(Component)]
pub struct EdgeController {
    pub project: String,
    pub id: String,
    pub source: String,
    pub target: String,
}

impl Default for EdgeController {
    fn default() -> Self {
        Self {
            project: "Unknown".to_string(),
            id: "1234".to_string(),
            source: "1234".to_string(),
            target: "1234".to_string(),
        }
    }
}

impl fmt::Display for EdgeController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Edge created ( project = {:?}, id = {:?}, source = {:?}, target = {:?} )",
            self.project, self.id, self.source, self.target
        )
    }
}

fn run_edge_controller(
    mut _napkin: ResMut<NapkinSettings>,
) {
    
}

fn edge_spawner(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut napkin: ResMut<NapkinSettings>,
    existing_nodes: Query<(&mut Transform, &mut NodeController)>,
    existing_edges: Query<&mut EdgeController>,
    mut ev_response: EventReader<TypedResponse<Vec<NapkinEdge>>>,
) {
    for response in ev_response.read() {
        info!("Received edges from server");
        napkin.edges = response.to_vec();
    }
    let mut filtered_edges: Vec<&NapkinEdge> = napkin.edges.iter().collect();
    if let Some(selected_project) = &napkin.selected_project {
        if !selected_project.is_empty() {
            filtered_edges.retain(|&edge| edge.project == *selected_project);
        }
    }
    for (index, edge) in filtered_edges.iter().enumerate() {
        if existing_edges.iter().all(|existing_edge| existing_edge.id != edge.id) {
            info!("Adding edge of ID {}", edge.id);
            let mut start_point = Vec3::ZERO;

            let mut source_node = None;
            let mut target_node = None;
            for existing_node in existing_nodes.iter() {
                if existing_node.1.id == edge.source {
                    source_node = Some(existing_node);
                } else if existing_node.1.id == edge.target {
                    target_node = Some(existing_node);
                }
            }

            if source_node.is_some() && target_node.is_some() {
                let target_node = target_node.unwrap();
                let source_node = source_node.unwrap();
                let center = Vec3::new(
                    (target_node.0.translation.x + source_node.0.translation.x) / 2.0,
                    (target_node.0.translation.y + source_node.0.translation.y) / 2.0,
                    0.0,
                );
                let length = source_node.0.translation.distance(target_node.0.translation);
                let transform = Transform::from_translation(center);
                commands.spawn((
                        bevy::sprite::MaterialMesh2dBundle {
                            mesh: meshes.add(Rectangle::new(1.0, length)).into(),
                            transform,
                            material: materials.add(ColorMaterial::from(Color::WHITE)),
                            ..default()
                        },
                        EdgeController {
                            project: edge.project.clone(),
                            id: edge.id.clone(),
                            source: edge.source.clone(),
                            target: edge.target.clone(),
                        },
                        RigidBody::Dynamic,
                        GravityScale(0.0),
                        Collider::ball(10.0),
                        CollisionGroups::new(Group::GROUP_13, Group::GROUP_4),
                        SolverGroups::new(Group::GROUP_13, Group::GROUP_4),
                ));    
            }
        }
    }
}

fn handle_edge_physics(
    mut edges: Query<(&mut Transform, &mut EdgeController, &mut Mesh2dHandle)>,
    mut meshes: ResMut<Assets<Mesh>>,
    nodes: Query<(&mut Transform, &mut NodeController), Without<EdgeController>>,
) {
    for (mut transform, mut edge, mut mesh) in edges.iter_mut() {
        let source_node = nodes.iter().find(|(_, node)| node.id == edge.source).unwrap();
        let target_node = nodes.iter().find(|(_, node)| node.id == edge.target).unwrap();

        let source_translation = source_node.0.translation;
        let target_translation = target_node.0.translation;
        let center = Vec3::new(
                (target_translation.x + source_translation.x) / 2.0,
                (target_translation.y + source_translation.y) / 2.0,
                0.0,
            );
        let length = (
            (source_translation.x - target_translation.x).powi(2)
            + (source_translation.y - target_translation.y).powi(2)
        ).sqrt();
        let rotation = Quat::from_rotation_z(
            ((target_translation.y - source_translation.y) / (target_translation.x - source_translation.x)).atan() + 90.0_f32.to_radians()
        );
        transform.translation = center;
        transform.rotation = rotation;

        mesh.0 = meshes.add(Rectangle::new(1.0, length)).into();
    }
}

pub fn edge_destroyer(
    mut commands: Commands,
    napkin: ResMut<NapkinSettings>,
    existing_edges: Query<(Entity, &mut EdgeController)>,
) {
    if let Some(selected_project) = &napkin.selected_project {
        if !selected_project.is_empty() {
            let filtered_edges = napkin
                .edges
                .iter()
                .filter(|edge| edge.project == *selected_project)
                .collect::<Vec<_>>();
            for (entity, edge) in existing_edges.iter() {
                if !filtered_edges
                    .iter()
                    .any(|&filtered_edge| filtered_edge.id == edge.id)
                {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
