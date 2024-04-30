use bevy::{prelude::*, window::PrimaryWindow};
use bevy_http_client::prelude::TypedResponse;
use bevy_rapier3d::{
    dynamics::{GravityScale, RapierRigidBodyHandle, RigidBody},
    geometry::{Collider, CollisionGroups, Group, SolverGroups},
    pipeline::QueryFilter,
    plugin::RapierContext,
    rapier::{
        dynamics::{RigidBodyBuilder, RigidBodySet, RigidBodyType},
        geometry::{ColliderBuilder, InteractionGroups},
    },
    render::ColliderDebugColor,
};
use std::fmt;

use crate::{camera_controller::*, NapkinNode, NapkinSettings};

pub struct NodeControllerPlugin;

impl Plugin for NodeControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                run_node_controller,
                cast_ray,
                node_spawner,
                handle_node_click,
                // handle_node_physics,
            ),
        );
    }
}

#[derive(Component)]
pub struct HoveredNode;

#[derive(Component)]
pub struct NodeController {
    pub visible: bool,
    pub project: String,
    pub id: String,
    pub position: Vec3,
}

impl Default for NodeController {
    fn default() -> Self {
        Self {
            visible: true,
            project: "Unknown".to_string(),
            id: "1234".to_string(),
            position: Vec3::ZERO,
        }
    }
}

impl fmt::Display for NodeController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Node created ( project = {:?}, id = {:?} )",
            self.project, self.id,
        )
    }
}

pub fn run_node_controller(
    mut napkin: ResMut<NapkinSettings>,
    time: Res<Time>,
    mut node_set: ParamSet<(
        Query<(&GlobalTransform, &mut Transform, &mut NodeController), Without<Camera>>,
        Query<(&mut HoveredNode, &NodeController), Without<Camera>>,
    )>,
    camera: Query<&Transform, With<Camera>>,
) {
    let target = camera.single();
    for (global_transform, mut node_pos, mut node_controller) in node_set.p0().iter_mut() {
        let start = node_pos.translation;
        let forward = start - target.translation;
        node_pos.look_at(start + forward, Vec3::Y);
        // if node_controller.position != global_transform.translation() {
        // info!("Position has changed");
        // }
        node_controller.position = global_transform.translation();
    }

    let mut new_selected_nodes: Vec<NapkinNode> = Vec::new();
    for (_, node_controller) in node_set.p1().iter_mut() {
        new_selected_nodes.push(NapkinNode {
            project: node_controller.project.clone(),
            id: node_controller.id.clone(),
        });
    }
    napkin.hovered_nodes = Some(new_selected_nodes);
}

pub fn handle_node_physics(
    mut commands: Commands,
    mut node_set: ParamSet<(
        Query<
            (
                &GlobalTransform,
                &mut Transform,
                &mut NodeController,
                &RapierRigidBodyHandle,
            ),
            Without<Camera>,
        >,
        Query<(&mut HoveredNode, &NodeController), Without<Camera>>,
    )>,
    mut rapier_context: ResMut<RapierContext>,
    time: Res<Time>,
) {
    let delta_time = time.delta_seconds();
    let center = Vec3::ZERO;
    let attraction_strength = 2.0;
    let repulsion_strength = 1.0;
    let damping = 0.1;

    let nodes: Vec<(Vec3, String)> = node_set
        .p0()
        .iter()
        .map(|(global_transform, _, node_controller, _)| {
            (global_transform.translation(), node_controller.id.clone())
        })
        .collect();

    for (global_transform, _, node_controller, rigid_body_handle) in node_set.p0().iter_mut() {
        if let Some(rigid_body) = rapier_context.bodies.get_mut(rigid_body_handle.0) {
            let mut total_force = Vec3::ZERO;
            let mut force_count = 0;

            let direction_to_center = center - global_transform.translation();
            let distance_to_center = direction_to_center.length();
            if distance_to_center > 0.0 {
                let attraction_force = direction_to_center.normalize() * (attraction_strength / distance_to_center.max(1.0)) * delta_time;
                total_force += attraction_force;
                force_count += 1;
            }

            for (other_global_transform, other_node_id) in &nodes {
                if node_controller.id != *other_node_id {
                    let direction_to_other = *other_global_transform - global_transform.translation();
                    let distance = direction_to_other.length();
                    if distance > 0.0 && distance < 0.2 {
                        let repulsion_force = -direction_to_other.normalize() * (repulsion_strength / distance.max(0.1)) * delta_time;
                        total_force += repulsion_force;
                        force_count += 1;
                    }
                }
            }

            let velocity = rigid_body.linvel();
            let velocity_vec3 = Vec3::new(velocity.x, velocity.y, velocity.z);
            let damping_force = -velocity_vec3 * damping * delta_time;
            total_force += damping_force;
            force_count += 1;

            if force_count > 0 {
                let average_force = total_force / force_count as f32;
                rigid_body.add_force(average_force.into(), true);
            }
        }
    }
}

pub fn handle_node_click(
    mut napkin: ResMut<NapkinSettings>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    existing_hover: Query<&NodeController, With<HoveredNode>>,
) {
    if let Ok(node) = existing_hover.get_single() {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            let selected_nodes = napkin.selected_nodes.get_or_insert_with(Vec::new);
            selected_nodes.push(NapkinNode {
                project: node.project.clone(),
                id: node.id.clone(),
            });
            napkin.napkin_crosshair = crate::NapkinCrosshair {
                selected_type: crate::NapkinCrosshairSelectionTypes::Node,
                selected_id: Some(node.id.clone()),
            };
        }
    }
}

pub fn cast_ray(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    rapier_context: Res<RapierContext>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    existing_hover: Query<Entity, With<HoveredNode>>,
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
            QueryFilter::only_dynamic(),
        );

        if let Some((entity, _toi)) = hit {
            commands.entity(entity).insert(HoveredNode);
        }

        for entity in existing_hover.iter() {
            if let Some((hit_entity, _)) = hit {
                if entity != hit_entity {
                    commands.entity(entity).remove::<HoveredNode>();
                }
            } else {
                commands.entity(entity).remove::<HoveredNode>();
            }
        }
    }
}

pub fn node_spawner(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut napkin: ResMut<NapkinSettings>,
    existing_nodes: Query<&mut NodeController>,
    mut ev_response: EventReader<TypedResponse<Vec<NapkinNode>>>,
) {
    for response in ev_response.read() {
        info!("Got response from server");
        napkin.is_connected = true;
        napkin.nodes = response.to_vec();
        for node in napkin.nodes.iter() {
            if existing_nodes
                .iter()
                .all(|existing_node| existing_node.id != node.id)
            {
                info!("Adding node of ID {}", node.id);
                let start_point = Vec3::new(
                    rand::random::<f32>() * 10. - 5.,
                    rand::random::<f32>() * 1. - 0.5,
                    rand::random::<f32>() * 10. - 5.,
                    // if node.id == "018d41d7-9f8e-0e88-5a19-8ccc64bcbfe6" { 5. } else { -3. },
                    // if node.id == "018d41d7-9f8e-0e88-5a19-8ccc64bcbfe6" { 3. } else { -2. },
                    // if node.id == "018d41d7-9f8e-0e88-5a19-8ccc64bcbfe6" { -5. } else { 5. },
                );
                let transform = Transform::from_translation(start_point);
                commands
                    .spawn((
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(Circle { radius: 0.1 })),
                            // mesh: meshes.add(Mesh::from(Sphere { radius: 0.1 })),
                            material: materials.add(StandardMaterial {
                                base_color: Color::WHITE,
                                unlit: true,
                                ..Default::default()
                            }),
                            transform,
                            ..Default::default()
                        },
                        NodeController {
                            project: node.project.clone(),
                            id: node.id.clone(),
                            position: start_point,
                            ..Default::default()
                        },
                        RigidBody::Dynamic,
                        GravityScale(0.0),
                        Collider::ball(0.1),
                    ))
                    .insert(CollisionGroups::new(Group::GROUP_13, Group::GROUP_4))
                    .insert(SolverGroups::new(Group::GROUP_3, Group::GROUP_11));

                // .insert(CollisionGroups::new(0b1101.into(), 0b0100.into()))
                // .insert(SolverGroups::new(0b0011.into(), 0b1011.into()));
            }
        }
    }
}
