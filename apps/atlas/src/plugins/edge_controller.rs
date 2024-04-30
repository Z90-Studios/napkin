use bevy::{prelude::*, window::PrimaryWindow};
use bevy_http_client::prelude::TypedResponse;
use bevy_rapier3d::{
    dynamics::{RapierRigidBodyHandle, RigidBody},
    geometry::{Collider, CollisionGroups, Group, SolverGroups},
    pipeline::QueryFilter,
    plugin::RapierContext,
};
use std::fmt;

use crate::{NapkinEdge, NapkinNode, NapkinSettings};

use super::node_controller::NodeController;

pub struct EdgeControllerPlugin;

impl Plugin for EdgeControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, run_edge_controller);
    }
}

#[derive(Component)]
pub struct HoveredEdge;

#[derive(Component)]
pub struct EdgeController {
    pub visible: bool,
    pub project: String,
    pub id: String,
    pub source: String,
    pub target: String,
    pub position: Vec3,
}

impl Default for EdgeController {
    fn default() -> Self {
        Self {
            visible: true,
            project: "Unknown".to_string(),
            id: "1234".to_string(),
            source: "1234".to_string(),
            target: "1234".to_string(),
            position: Vec3::ZERO,
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

pub fn run_edge_controller(
    mut napkin: ResMut<NapkinSettings>,
    mut data_set: ParamSet<(
        Query<
            (
                &GlobalTransform,
                &mut Transform,
                &mut NodeController,
                &RapierRigidBodyHandle,
            ),
            Without<Camera>,
        >,
        Query<(&mut Transform, &EdgeController), Without<Camera>>,
    )>,
    mut selected_edges: Query<(&mut HoveredEdge, &EdgeController), Without<Camera>>,
    // camera: Query<&Transform, With<Camera>>,
) {
    let nodes: Vec<(Vec3, String)> = data_set
        .p0()
        .iter()
        .map(|(global_transform, _, node_controller, _)| {
            (global_transform.translation(), node_controller.id.clone())
        })
        .collect();
    for (mut edge_pos, edge_controller) in data_set.p1().iter_mut() {
        let source_node_id = edge_controller.source.clone();
        let target_node_id = edge_controller.target.clone();
        let source = nodes
            .iter()
            .find(|(_, id)| id == &source_node_id)
            .map(|(pos, _)| *pos)
            .unwrap_or(Vec3::ZERO);

        let target = nodes
            .iter()
            .find(|(_, id)| id == &target_node_id)
            .map(|(pos, _)| *pos)
            .unwrap_or(Vec3::ZERO);

        let mut line_transform = Transform::from_translation(Vec3::ZERO);

        for (mut node_pos, node_id) in &nodes {
            if (node_id == &source_node_id && node_pos != source)
                || (node_id == &target_node_id && node_pos != target)
            {
                let direction = (target - source).normalize();
                let line_length = (source - target).length();
                line_transform = Transform::from_translation((source + target) / 2.)
                    .looking_to(((source + target) / 2.).cross(target), direction);
                edge_pos.translation = line_transform.translation;
                edge_pos.rotation = line_transform.rotation;
                edge_pos.scale = line_transform.scale;
            }
        }
    }

    let mut new_selected_edges: Vec<NapkinEdge> = Vec::new();
    for (_, edge_controller) in selected_edges.iter_mut() {
        new_selected_edges.push(NapkinEdge {
            project: edge_controller.project.clone(),
            id: edge_controller.id.clone(),
            source: edge_controller.source.clone(),
            target: edge_controller.target.clone(),
        });
    }
    napkin.hovered_edges = Some(new_selected_edges);
}

pub fn cast_ray(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    rapier_context: Res<RapierContext>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    existing_hover: Query<Entity, With<HoveredEdge>>,
) {
    let window = windows.single();

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    for (camera, camera_transform) in &cameras {
        // Compute ray from mouse position
        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
            return;
        };

        // Cast the ray
        let hit = rapier_context.cast_ray(
            ray.origin,
            ray.direction.into(),
            f32::MAX,
            true,
            QueryFilter::only_fixed(),
        );

        if let Some((entity, _toi)) = hit {
            commands.entity(entity).insert(HoveredEdge);
        }

        for entity in existing_hover.iter() {
            if let Some((hit_entity, _)) = hit {
                if entity != hit_entity {
                    commands.entity(entity).remove::<HoveredEdge>();
                }
            } else {
                commands.entity(entity).remove::<HoveredEdge>();
            }
        }
    }
}

pub fn edge_spawner(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut napkin: ResMut<NapkinSettings>,
    existing_nodes: Query<&mut NodeController>,
    existing_edges: Query<&mut EdgeController>,
    mut ev_response: EventReader<TypedResponse<Vec<NapkinEdge>>>,
) {
    for response in ev_response.read() {
        napkin.is_connected = true;
        napkin.edges = response.to_vec();
    }
    for edge in napkin.edges.iter() {
        if existing_edges
            .iter()
            .all(|existing_edge| existing_edge.id != edge.id)
        {
            info!("Adding edge of ID {}", edge.id);
            let mut source_node = None;
            let mut target_node = None;
            for existing_node in existing_nodes.iter() {
                if existing_node.id == edge.source {
                    source_node = Some(existing_node);
                } else if existing_node.id == edge.target {
                    target_node = Some(existing_node);
                }
            }
            if let Some(source) = source_node {
                if let Some(target) = target_node {
                    // Had to do some funky math here
                    // Cuboid spawns long-wise along the Y-axis (top to bottom)
                    // The following transform will make it "look at" the sky (inverse of the path between nodes)
                    // all so that the cuboid will be oriented in the right direction
                    let direction = (target.position - source.position).normalize();
                    let line_length = (source.position - target.position).length();
                    let line_transform =
                        Transform::from_translation((source.position + target.position) / 2.)
                            .looking_to(
                                ((source.position + target.position) / 2.).cross(target.position),
                                direction,
                            );

                    let line_mesh = meshes.add(
                        Mesh::from(Capsule3d {
                            radius: 0.02,
                            half_length: line_length / 2.,
                        })
                        .transformed_by(line_transform),
                    );
                    let line_material = materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        unlit: true,
                        ..Default::default()
                    });
                    commands
                        .spawn((
                            PbrBundle {
                                mesh: line_mesh,
                                material: line_material,
                                ..Default::default()
                            },
                            EdgeController {
                                project: edge.project.clone(),
                                id: edge.id.clone(),
                                source: edge.source.clone(),
                                target: edge.target.clone(),
                                position: line_transform.translation,
                                ..Default::default()
                            },
                            RigidBody::Fixed,
                            Collider::capsule(source.position, target.position, 0.03),
                        ))
                        .insert(CollisionGroups::new(Group::GROUP_14, Group::GROUP_5))
                        .insert(SolverGroups::new(Group::GROUP_8, Group::GROUP_18));
                } else {
                    // todo!("Implement UI element that states that target was not found.");
                }
            } else {
                // todo!("Implement UI element that states that source was not found.");
            }
        }
    }
}
