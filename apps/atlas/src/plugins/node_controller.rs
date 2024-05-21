use bevy::{input::keyboard::KeyboardInput, prelude::*, window::{CursorGrabMode, PrimaryWindow}};
use bevy_egui::{
    egui::{self, Color32, CursorIcon},
    EguiContexts,
};
use bevy_http_client::prelude::TypedResponse;
use bevy_rapier3d::{
    dynamics::{GravityScale, RapierRigidBodyHandle, RigidBody},
    geometry::{Collider, CollisionGroups, Group, SolverGroups},
    pipeline::QueryFilter,
    plugin::RapierContext, rapier::geometry::InteractionGroups,
};
use std::fmt;

use crate::{NapkinNode, NapkinSettings};

pub struct NodeControllerPlugin;

impl Plugin for NodeControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                run_node_controller,
                node_tooltip,
                cast_ray,
                node_spawner,
                node_destroyer,
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
    mut query: Query<(&mut Transform, &NodeController)>,
    time: Res<Time>,
    napkin: Res<NapkinSettings>,
) {
    let center = Vec3::ZERO; // Define the center of attraction
    let nodes = query
        .iter_mut()
        .map(|(transform, node_controller)| (transform.translation, node_controller))
        .collect::<Vec<_>>();
    let mut velocities = vec![Vec3::ZERO; nodes.len()];

    // Precompute time delta for efficiency
    let delta_time = time.delta_seconds();

    // Simple physics: repulsion between nodes and attraction to the center
    for i in 0..nodes.len() {
        let node_position = nodes[i].0;
        // Attraction to the center
        let center_direction = center - node_position;
        let center_distance = center_direction.length();
        if center_distance > 0.0 {
            let center_force_magnitude = center_distance * 0.2; // Attractive force proportional to distance
            velocities[i] += center_direction.normalize() * center_force_magnitude * delta_time;
        }

        // Repulsion between nodes, lessened if the node is connected with an edge
        for j in 0..nodes.len() {
            if i != j {
                let direction = node_position - nodes[j].0;
                let distance = direction.length();
                let connected = napkin.edges.iter().any(|edge| 
                    (edge.source == nodes[i].1.id && edge.target == nodes[j].1.id) ||
                    (edge.target == nodes[i].1.id && edge.source == nodes[j].1.id)
                );
                let repulsion_factor = if connected { 0.3 } else { 0.6 };
                if distance > 0.0 {
                    let force_magnitude = repulsion_factor / distance.max(0.6); // Repulsive force inversely proportional to distance, adjusted by connection
                    velocities[i] += direction.normalize() * force_magnitude * delta_time;
                }
            }
        }

        // // Additional attraction between nodes with matching edges
        // let edges = &napkin.edges;
        // for edge in edges {
        //     if edge.source == nodes[i].1.id || edge.target == nodes[i].1.id {
        //         let target_index = nodes.iter().position(|(_, nc)| nc.id == edge.source || nc.id == edge.target);
        //         if let Some(target_index) = target_index {
        //             let target_position = nodes[target_index].0;
        //             let attraction_direction = target_position - node_position;
        //             let attraction_distance = attraction_direction.length();
        //             if attraction_distance > 1. {
        //                 let attraction_force_magnitude = attraction_distance * attraction_distance.min(1.); // Attractive force proportional to distance
        //                 velocities[i] += attraction_direction.normalize() * attraction_force_magnitude * delta_time;
        //             }
        //         }
        //     }
        // }
    }

    // Update positions based on velocities and apply damping to simulate friction or air resistance
    let damping_factor = 0.85; // Reduce velocity by 15% each frame to simulate energy loss
    for (i, (mut transform, _)) in query.iter_mut().enumerate() {
        velocities[i] *= damping_factor; // Apply damping
        transform.translation += velocities[i];
        if velocities[i].length() < 0.1 { // Threshold to stop movement
            velocities[i] = Vec3::ZERO;
        }
    }
}

pub fn handle_node_click(
    mut napkin: ResMut<NapkinSettings>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    existing_hover: Query<&NodeController, With<HoveredNode>>,
) {
    if let Ok(node) = existing_hover.get_single() {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            let selected_nodes = napkin.selected_nodes.get_or_insert_with(Vec::new);
            if !keyboard_input.pressed(KeyCode::ShiftLeft) && !keyboard_input.pressed(KeyCode::ShiftRight) {
                selected_nodes.clear();
            }
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
    mut contexts: EguiContexts,
) {
    let ctx = contexts.ctx_mut();
    let window = windows.single();

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    for (camera, camera_transform) in &cameras {
        // Compute ray from mouse position
        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
            return;
        };
        
        // Don't register hits when rotating camera
        if window.cursor.grab_mode == CursorGrabMode::Locked {
            return;
        }

        // Cast the ray
        let hit = rapier_context.cast_ray(
            ray.origin,
            ray.direction.into(),
            f32::MAX,
            true,
            QueryFilter::new().groups(CollisionGroups::new(Group::ALL, Group::GROUP_13)),
        );


        if let Some((entity, _toi)) = hit {
            commands.entity(entity).insert(HoveredNode);
            ctx.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
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

pub fn node_tooltip(
    mut napkin: ResMut<NapkinSettings>,
    windows: Query<&mut Window>,
    mut contexts: EguiContexts,
) {
    let ctx = contexts.ctx_mut();

    if let (Some(mouse_position), window_height) = (
        windows.single().cursor_position(),
        windows.single().height(),
    ) {
        let node_tooltip_frame = egui::Frame {
            fill: egui::Color32::from_black_alpha((255. * 0.9) as u8),
            stroke: egui::Stroke::NONE,
            ..egui::Frame::none()
        };
        // TODO: Make it so you can lock a tooltip and click on things
        let offset_position = [
            mouse_position.x + 6.,
            (mouse_position.y - 6.) - window_height,
        ];
        if let Some(hovered_nodes) = &napkin.hovered_nodes {
            if !hovered_nodes.is_empty() {
                egui::Window::new("Node Details")
                    .id(egui::Id::new("node_tooltip"))
                    .title_bar(false)
                    .resizable(false)
                    .frame(node_tooltip_frame)
                    .anchor(egui::Align2::LEFT_BOTTOM, offset_position)
                    .show(ctx, |ui| {
                        for node in hovered_nodes {
                            let _project = &napkin
                                .projects
                                .iter()
                                .find(|&project| project.id == node.project);
                            if let Some(project) = _project {
                                ui.label(format!("@{}/{}", project.scope, project.name))
                                    .highlight();
                            }
                            ui.label(format!("Node ID: {}", node.id));
                            ui.label("Metadata");
                            let metadata_color = Color32::from_rgb(180, 180, 220);
                            for metadata in napkin
                                .node_metadata
                                .iter()
                                .filter(|&node_metadata| node_metadata.owner_id == node.id)
                            {
                                egui::Frame::default()
                                    .stroke(egui::Stroke::new(0.5, metadata_color))
                                    .rounding(ui.visuals().widgets.noninteractive.rounding)
                                    .inner_margin(4.)
                                    .show(ui, |ui| {
                                        ui.style_mut().wrap = Some(false);
                                        ui.label(
                                            egui::RichText::new(format!("{}", metadata.name))
                                                .color(metadata_color),
                                        );
                                    });
                            }
                        }
                    });
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
        info!("Received nodes from server");
        napkin.is_connected = true;
        napkin.nodes = response.to_vec();
    }
    let mut filtered_nodes: Vec<&NapkinNode> = napkin.nodes.iter().collect();
    if let Some(selected_project) = &napkin.selected_project {
        if !selected_project.is_empty() {
            filtered_nodes = filtered_nodes
                .iter()
                .filter(|&&node| node.project == *selected_project)
                .cloned()
                .collect();
        }
    }
    fn calculate_balanced_start_point(index: usize, total_nodes: usize) -> Vec3 {
        let angle = 2.0 * std::f32::consts::PI * (index as f32) / (total_nodes as f32);
        let radius = total_nodes as f32 * 0.3;
        Vec3::new(
            radius * angle.cos(),
            0.5 * angle.sin() + 0.5,
            radius * angle.sin(),
        )
    }

    let total_nodes = filtered_nodes.len();
    for (index, node) in filtered_nodes.iter().enumerate() {
        if existing_nodes
            .iter()
            .all(|existing_node| existing_node.id != node.id)
        {
            info!("Adding node of ID {}", node.id);
            let start_point = calculate_balanced_start_point(index, total_nodes);
            let transform = Transform::from_translation(start_point);
            commands
                .spawn((
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(Circle { radius: 0.1 })),
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
                // Assign the node to collision group 13 and allow it to collide with group 4
                .insert(CollisionGroups::new(Group::GROUP_13, Group::GROUP_4))
                // Assign the node to solver group 3 and allow interaction with solver group 11
                .insert(SolverGroups::new(Group::GROUP_13, Group::GROUP_4));
        }
    }
}

pub fn node_destroyer(
    mut commands: Commands,
    mut napkin: ResMut<NapkinSettings>,
    existing_nodes: Query<(Entity, &mut NodeController)>,
) {
    if let Some(selected_project) = &napkin.selected_project {
        if !selected_project.is_empty() {
            let filtered_nodes = napkin
                .nodes
                .iter()
                .filter(|node| node.project == *selected_project)
                .collect::<Vec<_>>();
            for (entity, node) in existing_nodes.iter() {
                if !filtered_nodes
                    .iter()
                    .any(|&filtered_node| filtered_node.id == node.id)
                {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
