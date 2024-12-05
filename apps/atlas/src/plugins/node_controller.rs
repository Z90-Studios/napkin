use bevy::{prelude::*, window::{CursorGrabMode, PrimaryWindow}};
use bevy_egui::{
    egui::{self, Color32, CursorIcon},
    EguiContexts,
};
use bevy_http_client::{prelude::{TypedRequest, TypedResponse}, HttpClient};
use bevy_rapier2d::{
    dynamics::{GravityScale, RigidBody},
    geometry::{Collider, CollisionGroups, Group, SolverGroups},
    pipeline::QueryFilter,
    plugin::RapierContext,
};
use std::fmt;

use crate::{NapkinEdits, NapkinNode, NapkinSettings, OccupiedScreenSpace};

use super::camera_controller::CameraController;

pub struct NodeControllerPlugin;

impl Plugin for NodeControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                run_node_controller,
                node_spawner,
                node_destroyer,
                cast_ray,
                handle_node_physics,
                handle_click,
                save_node,
                apply_response,
            )
        );
    }
}

#[derive(Component)]
pub struct HoveredNode;

#[derive(Component)]
pub struct NodeController {
    pub project: String,
    pub id: String,
}

impl Default for NodeController {
    fn default() -> Self {
        Self {
            project: "Unknown".to_string(),
            id: "1234".to_string(),
        }
    }
}

impl fmt::Display for NodeController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Node created ( project = {:?}, id = {:?} )",
            self.project, self.id
        )
    }
}

impl From<&NodeController> for NapkinNode {
    fn from(node: &NodeController) -> NapkinNode {
        NapkinNode {
            id: node.id.clone(),
            project: node.project.clone(),
        }
    }
}

fn run_node_controller(
    mut napkin: ResMut<NapkinSettings>,
    _time: Res<Time>,
    mut node_set: ParamSet<(
        Query<(&GlobalTransform, &mut Transform, &mut NodeController), Without<Camera>>,
        Query<(&mut HoveredNode, &NodeController), Without<Camera>>,
    )>,
) {
    let mut new_hovered_nodes: Vec<NapkinNode> = Vec::new();
    for (_, node_controller) in node_set.p1().iter_mut() {
        new_hovered_nodes.push(NapkinNode {
            project: node_controller.project.clone(),
            id: node_controller.id.clone(),
        });
    }
    napkin.hovered_nodes = Some(new_hovered_nodes);
}

fn node_spawner(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut napkin: ResMut<NapkinSettings>,
    existing_nodes: Query<&mut NodeController>,
    mut ev_response: EventReader<TypedResponse<Vec<NapkinNode>>>,
) {
    for response in ev_response.read() {
        info!("Received nodes from server");
        napkin.nodes = response.to_vec();
    }
    let mut filtered_nodes: Vec<&NapkinNode> = napkin.nodes.iter().collect();
    if let Some(selected_project) = &napkin.selected_project {
        if !selected_project.is_empty() {
            let filtered_edges: Vec<String> = napkin.
                edges
                .iter()
                .filter(|e| e.project == selected_project.clone())
                .flat_map(|e| vec![e.source.clone(), e.target.clone()])
                .into_iter()
                .collect();
            filtered_nodes.retain(|&node| node.project == *selected_project || filtered_edges.contains(&node.id));
        }
    }
    fn calculate_balanced_start_point(index: usize, total_nodes: usize) -> Vec2 {
        let angle = 2.0 * std::f32::consts::PI * (index as f32) / (total_nodes as f32);
        let radius = total_nodes as f32 * 30.0;
        Vec2::new(
            radius * angle.cos(),
            radius * angle.sin(),
        )
    }

    let node_size: f32 = 6.0;

    let total_nodes = filtered_nodes.len();
    for (index, node) in filtered_nodes.iter().enumerate() {
        if existing_nodes.iter().all(|existing_node| existing_node.id != node.id) {
            info!("Adding node of ID {}", node.id);
            let start_point = calculate_balanced_start_point(index, total_nodes);
            let transform = Transform::from_translation(start_point.extend(100.));
            commands.spawn((
                    bevy::sprite::MaterialMesh2dBundle {
                        mesh: meshes.add(Circle::new(node_size)).into(),
                        transform,
                        material: materials.add(ColorMaterial::from(Color::WHITE)),
                        ..default()
                    },
                    NodeController {
                        project: node.project.clone(),
                        id: node.id.clone(),
                    },
                    RigidBody::Dynamic,
                    GravityScale(0.0),
                    Collider::ball(node_size),
                    CollisionGroups::new(Group::GROUP_13, Group::GROUP_3),
                    SolverGroups::new(Group::GROUP_13, Group::GROUP_3),
            ));
        }
    }
}

pub fn node_destroyer(
    mut commands: Commands,
    napkin: ResMut<NapkinSettings>,
    existing_nodes: Query<(Entity, &mut NodeController)>,
) {
    if let Some(selected_project) = &napkin.selected_project {
        if !selected_project.is_empty() {
            let filtered_edges: Vec<String> = napkin.
                edges
                .iter()
                .filter(|e| e.project == selected_project.clone())
                .flat_map(|e| vec![e.source.clone(), e.target.clone()])
                .into_iter()
                .collect();
            let filtered_nodes = napkin
                .nodes
                .iter()
                .filter(|node| node.project == *selected_project || filtered_edges.contains(&node.id))
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

pub fn handle_drag(
    mut hovered_node: Query<&mut Transform, With<HoveredNode>>,
) {}

pub fn handle_click(
    hovered_nodes: Query<(&HoveredNode, &NodeController)>,
    camera_controller: Query<&CameraController>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut napkin: ResMut<NapkinSettings>,
    occupied_screen_space: Res<OccupiedScreenSpace>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.single();
    let mouse_border_offset = 4.0;
    if napkin.context_menu_open {
        return;
    }
    if let (Some(cursor_position), window_height, window_width) = (
        window.cursor_position(),
        window.height(),
        window.width(),
    ) {
        if cursor_position.x < occupied_screen_space.left + mouse_border_offset
            || cursor_position.x
                > (window_width - occupied_screen_space.right - mouse_border_offset)
            || cursor_position.y
                > (window_height - occupied_screen_space.bottom - mouse_border_offset)
            || cursor_position.y < (occupied_screen_space.top + mouse_border_offset)
        {
            return;
        }
    } else {
        return;
    }

    if mouse_button_input.just_pressed(camera_controller.single().mouse_key_cursor_grab) {
        for (_, node) in &hovered_nodes {
            napkin.selected_node = Some(node.id.clone());
            return;
        }
        napkin.selected_node = None;
    }
}

pub fn cast_ray(
    mut commands: Commands,
    napkin: Res<NapkinSettings>,
    occupied_screen_space: Res<OccupiedScreenSpace>,
    windows: Query<&Window, With<PrimaryWindow>>,
    rapier_context: Res<RapierContext>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut camera_controller_query: Query<&mut CameraController>,
    mut nodes: Query<(Entity, &mut Handle<ColorMaterial>), With<NodeController>>,
    mut contexts: EguiContexts,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ctx = contexts.ctx_mut();
    let window = windows.single();
    let mut camera_controller = camera_controller_query.get_single_mut().unwrap();

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let mouse_border_offset = 4.0;
    if let (Some(cursor_position), window_height, window_width) = (
        window.cursor_position(),
        window.height(),
        window.width(),
    ) {
        if cursor_position.x < occupied_screen_space.left + mouse_border_offset
            || cursor_position.x
                > (window_width - occupied_screen_space.right - mouse_border_offset)
            || cursor_position.y
                > (window_height - occupied_screen_space.bottom - mouse_border_offset)
            || cursor_position.y < (occupied_screen_space.top + mouse_border_offset)
        {
            return;
        }
    }
    if napkin.context_menu_open {
        return;
    }

    for (camera, camera_transform) in &cameras {
        // Compute ray from mouse position
        let Some(point) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
            return;
        };
        
        // Don't register hits when rotating camera
        if window.cursor.grab_mode == CursorGrabMode::Locked {
            return;
        }

        let mut entity: Option<Entity> = None;
        // Cast the ray
        rapier_context.intersections_with_point(
            point,
            QueryFilter::new().groups(CollisionGroups::new(Group::GROUP_3, Group::GROUP_13)),
            |e| {
                // Callback called on each collider hit by the ray.
                entity = Some(e);
                commands.entity(e).insert(HoveredNode);
                camera_controller.enabled = false;
                // if nodes.contains(entity) {

                true // Return `false` instead if we want to stop searching for other hits.
            },
        );

        for (n_entity, color_material) in &mut nodes.iter() {
            let material = materials.get_mut(color_material).unwrap();
            if entity.is_some() {
                if n_entity == entity.unwrap() {
                    // TODO: Move to a CursorIconController
                    ctx.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                    material.color = Color::linear_rgb(66.0 / 255.0, 135.0 / 255.0, 245.0 / 255.0);
                }
            } else {
                commands.entity(n_entity).remove::<HoveredNode>();
                material.color = Color::WHITE;
                camera_controller.enabled = true;
            }
        }
        // if let Some((entity, _toi)) = hit {
        //     commands.entity(entity).insert(HoveredNode);
        //     ctx.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
        // }

        // for entity in existing_hover.iter() {
        //     if let Some((hit_entity, _)) = hit {
        //         if entity != hit_entity {
        //             commands.entity(entity).remove::<HoveredNode>();
        //         }
        //     } else {
        //         commands.entity(entity).remove::<HoveredNode>();
        //     }
        // }
    }
}

fn handle_node_physics(
    mut query: Query<(&mut Transform, &NodeController)>,
    time: Res<Time>,
    napkin: Res<NapkinSettings>,
) {
    let center = Vec3::ZERO; // Define center (vec3 for transforms)
    let nodes = query
        .iter_mut()
        .map(|(transform, node_controller)| (transform.translation, node_controller))
        .collect::<Vec<_>>();
    let mut velocities = vec![Vec3::ZERO; nodes.len()];

    let delta_time = time.delta_seconds();

    for i in 0..nodes.len() {
        let node_position = nodes[i].0;
        // Attraction to center
        let center_direction = center - node_position;
        let center_distance = center_direction.length();
        if center_distance > 0.0 {
            let center_force_magnitude = center_distance * 0.4;
            velocities[i] += center_direction.normalize() * center_force_magnitude * delta_time;
        }

        // Repulsion between nodes
        for j in 0..nodes.len() {
            if i != j {
                let direction = node_position - nodes[j].0;
                let distance = direction.length();
                let connected = napkin.edges.iter().any(|edge|
                    (edge.source == nodes[i].1.id && edge.target == nodes[j].1.id) ||
                    (edge.source == nodes[i].1.id && edge.source == nodes[j].1.id)
                );
                let repulsion_factor = if connected { 500.0 } else { 1000.0 };
                // Repulsive force inverse to distance
                let force_magnitude = repulsion_factor / distance.max(50.0);
                velocities[i] += direction.normalize() * force_magnitude * delta_time;    
            }
        }
    }

    // Update positions and apply damping to simulate friction
    let damping_factor = 1.2;
    for (i, (mut transform, _)) in query.iter_mut().enumerate() {
        velocities[i] *= damping_factor;
        if velocities[i].length() < 0.05 {
            velocities[i] = Vec3::ZERO;
        }
        transform.translation += velocities[i];
    }
}

pub fn save_node(
    mut napkin: ResMut<NapkinSettings>,
    mut edits: ResMut<NapkinEdits>,
    mut node_request: EventWriter<TypedRequest<NapkinNode>>,
) {
    if edits.save_node {
        let mut http_client = HttpClient::new();
        if edits.node.id.is_empty() {
            http_client = http_client.post(format!("{}/node", napkin.server_url));
        } else {
            http_client = http_client.put(format!("{}/node/{}", napkin.server_url, edits.node.id));
        }
        http_client = http_client.json(&edits.node);
        node_request.send(
            http_client.with_type::<NapkinNode>(),
        );
        edits.save_node = false;
    }
}

pub fn apply_response(
    mut napkin: ResMut<NapkinSettings>,
    mut edits: ResMut<NapkinEdits>,
    mut node_response: EventReader<TypedResponse<NapkinNode>>,
) {
    for response in node_response.read() {
        info!("Received updated node from server");
        let node = NapkinNode {
            id: response.id.clone(),
            project: response.project.clone(),
        };
        edits.node = node.clone();
        let mut exists = false;
        for n in napkin.nodes.iter_mut() {
            if n.id == node.id {
                exists = true;
                *n = node.clone();
            }
        }
        if exists == false {
            napkin.nodes.push(node.clone());
        }
    }
}
