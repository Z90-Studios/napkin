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

use super::{camera_controller::{self, CameraController}, node_controller::NodeController};

pub struct EdgeControllerPlugin;

impl Plugin for EdgeControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
                run_edge_controller,
                edge_spawner,
                edge_destroyer,
                handle_edge_physics,
                cast_ray,
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
    pub orig_length: f32,
}

impl Default for EdgeController {
    fn default() -> Self {
        Self {
            project: "Unknown".to_string(),
            id: "1234".to_string(),
            source: "1234".to_string(),
            target: "1234".to_string(),
            orig_length: 10.0,
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
                let rotation = Quat::from_rotation_z(
                ((target_node.0.translation.y - source_node.0.translation.y) / (target_node.0.translation.x - source_node.0.translation.x)).atan() + 90.0_f32.to_radians()
                );

                let length = source_node.0.translation.distance(target_node.0.translation);
                let mut transform = Transform::from_translation(center);
                transform.rotation = rotation;
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
                            orig_length: length,
                        },
                        RigidBody::Dynamic,
                        GravityScale(0.0),
                        Collider::cuboid(3.0, length / 2.0),
                        CollisionGroups::new(Group::GROUP_14, Group::GROUP_4),
                        SolverGroups::new(Group::GROUP_14, Group::GROUP_4),
                ));    
            }
        }
    }
}

fn cast_ray(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    rapier_context: Res<RapierContext>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut camera_controller_query: Query<&mut CameraController>,
    mut edges: Query<(Entity, &mut Handle<ColorMaterial>), With<EdgeController>>,
    mut contexts: EguiContexts,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ctx = contexts.ctx_mut();
    let window = windows.single();
    let mut camera_controller = camera_controller_query.get_single_mut().unwrap();

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    for (camera, camera_transform) in &cameras {
        let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
            return;
        };

        if window.cursor.grab_mode == CursorGrabMode::Locked {
            return;
        }

        let mut node_match = false;
        rapier_context.intersections_with_point(
            point,
            QueryFilter::new().groups(CollisionGroups::new(Group::GROUP_3, Group::GROUP_13)),
            |e| {
                node_match = true;

                true
            },
        );

        let mut entity: Option<Entity> = None;

        if !node_match {
            rapier_context.intersections_with_point(
                point,
                QueryFilter::new().groups(CollisionGroups::new(Group::GROUP_4, Group::GROUP_14)),
                |e| {
                    entity = Some(e);
                    commands.entity(e).insert(HoveredEdge);
                    camera_controller.enabled = false;
    
                    true
                },
            );
        }

        for (n_entity, color_material) in &mut edges.iter() {
            let material = materials.get_mut(color_material).unwrap();
            if entity.is_some() {
                if n_entity == entity.unwrap() {
                    ctx.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                    material.color = Color::linear_rgb(66.0 / 255.0, 135.0 / 255.0, 245.0 / 255.0);
                }
            } else {
                commands.entity(n_entity).remove::<HoveredEdge>();
                material.color = Color::WHITE;
                camera_controller.enabled = true;
            }
        }
    }
}

fn handle_edge_physics(
    mut commands: Commands,
    mut edges: Query<(Entity, &mut Transform, &mut EdgeController, &mut Mesh2dHandle)>,
    mut meshes: ResMut<Assets<Mesh>>,
    nodes: Query<(&mut Transform, &mut NodeController), Without<EdgeController>>,
) {
    for (mut entity, mut transform, mut edge, mut mesh) in edges.iter_mut() {
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
        if length / edge.orig_length != 1.0 {
            transform.scale = Vec3::new(1.0, length / edge.orig_length, 1.0);

            //commands.entity(entity).remove::<Collider>();
            //commands.entity(entity).insert(Collider::cuboid(1.0, length));
        }       
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
