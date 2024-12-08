use bevy::{prelude::*, sprite::Mesh2dHandle, window::{CursorGrabMode, PrimaryWindow}};
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
use kdtree::KdTree;

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
                update_controller,
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
    napkin: ResMut<NapkinSettings>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    nodes: Query<(Entity, &mut NodeController, &mut Handle<ColorMaterial>, Option<&HoveredNode>), With<NodeController>>,
    mut contexts: EguiContexts,
) {
    let ctx = contexts.ctx_mut();
    let selected_node = napkin.selected_node.clone().unwrap_or("".to_string());
    for (_, node_controller, color_material, hovered) in &mut nodes.iter() {
        let selected = selected_node == node_controller.id;
        let material = materials.get_mut(color_material).unwrap();
        let color = if selected {
            Color::linear_rgb(
                64.0 / 255.0,
                182.0 / 255.0,
                60.0 / 255.0
            )
        } else {
            Color::WHITE
        };
        let highlight_color = if selected {
            Color::linear_rgb(
                65.0 / 255.0,
                159.0 / 255.0,
                153.0 / 255.0
            )
        } else {
            Color::linear_rgb(
                66.0 / 255.0,
                135.0 / 255.0,
                245.0 / 255.0
            )
        };
        if hovered.is_some() {
            ctx.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
            material.color = highlight_color;
        } else {
            material.color = color;
        }
    }
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
        const GOLDEN_RATIO: f32 = 1.61803398875;
        const GOLDEN_ANGLE: f32 = std::f32::consts::PI * (3.0 - GOLDEN_RATIO);

        let angle = index as f32 * GOLDEN_ANGLE;

        let radius = 10.0 * (index as f32).sqrt();

        Vec2::new(
            radius * angle.cos(),
            radius * angle.sin(),
        )
        // let angle = 2.0 * std::f32::consts::PI * (index as f32) / (total_nodes as f32);
        // let radius = 5.0;
        // Vec2::new(
        //     radius * angle.cos(),
        //     radius * angle.sin(),
        // )
    }

    let node_size: f32 = 6.0;

    let total_nodes = filtered_nodes.len();
    for (index, node) in filtered_nodes.iter().enumerate() {
        if existing_nodes.iter().all(|existing_node| existing_node.id != node.id) {
            info!("Adding node of ID {}", node.id);
            let start_point = calculate_balanced_start_point(index, total_nodes);
            let transform = Transform::from_translation(start_point.extend(10.));

            let mut mesh = meshes.add(Circle::new(node_size)).into();
            let mut material = materials.add(ColorMaterial::from(Color::WHITE));
            if napkin.selected_project.is_some() && Some(node.project.clone()) != napkin.selected_project {
                mesh = meshes.add(Rectangle::new(10.0, 10.0)).into();
                material = materials.add(ColorMaterial::from(Color::linear_rgb(0.8, 0.8, 0.8)));
            }

            commands.spawn((
                    bevy::sprite::MaterialMesh2dBundle {
                        mesh,
                        transform,
                        material,
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
    mut nodes: Query<(Entity, &NodeController, &mut Handle<ColorMaterial>), With<NodeController>>,
    hovered_nodes: Query<Entity, With<HoveredNode>>,
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

                true // Return `false` instead if we want to stop searching for other hits.
            },
        );

        if let Some(e) = entity {
            camera_controller.enabled = false;
            for entity in hovered_nodes.iter() {
                if entity != e {
                    commands.entity(entity).remove::<HoveredNode>();
                }
            }
            commands.entity(e).insert(HoveredNode);
        } else {
            for entity in hovered_nodes.iter() {
                commands.entity(entity).remove::<HoveredNode>();
            }
            return;
        }

    }
}

fn handle_node_physics(
    mut query: Query<(&mut Transform, &NodeController)>,
    time: Res<Time>,
    napkin: Res<NapkinSettings>,
) {
    let nodes = query
        .iter_mut()
        .map(|(transform, node_controller)| (transform.translation, node_controller))
        .collect::<Vec<_>>();
    let area = (nodes.len() as f32).sqrt() * (3.0);
    let k = (area / nodes.len() as f32).sqrt();
    let cooling_factor: f32 = 0.1;
    let mut rng = rand::thread_rng();
    let mut velocities = vec![Vec3::ZERO; nodes.len()];

    let global_repulsion_factor = 90.0;

    let mut edge_weight: Vec<usize> = vec![0 as usize; nodes.len()];

    // Attraction
    for edge in napkin.edges.iter() {
        if let (Some((i, source)), Some((j, target))) = (
            nodes.iter().enumerate().find(|(_, n)| n.1.id == edge.source),
            nodes.iter().enumerate().find(|(_, n)| n.1.id == edge.target)
        ) {
            let dx = source.0.x - target.0.x;
            let dy = source.0.y - target.0.y;
            let distance = (dx * dx + dy * dy).sqrt();
            edge_weight[i] += 1;
            edge_weight[j] += 1;
            if distance > 0.0 {
                let attractive_force = (distance - 100.0) / distance;
                velocities[i].x -= dx / distance * attractive_force;
                velocities[i].y -= dy / distance * attractive_force;
                velocities[j].x += dx / distance * attractive_force;
                velocities[j].y += dy / distance * attractive_force;
            }
        }
    }

    // Repulsion
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            let iw = edge_weight[i];
            let jw = edge_weight[j];
            let dx = nodes[i].0.x - nodes[j].0.x;
            let dy = nodes[i].0.y - nodes[j].0.y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > 0.0 {
                //let repulsive_force = k * k / distance;
                let repulsive_force = global_repulsion_factor / (distance * distance);
                let weight_factor = 1.0 * (iw + jw + 1) as f32;
                velocities[i].x += dx / distance * repulsive_force * weight_factor;
                velocities[i].y += dy / distance * repulsive_force * weight_factor;
                velocities[j].x -= dx / distance * repulsive_force * weight_factor;
                velocities[j].y -= dy / distance * repulsive_force * weight_factor;
            }
        }
    }


    // Gravity
    for i in 0..nodes.len() {
        let ew = edge_weight[i];
        velocities[i] += -nodes[i].0.normalize() * (ew as f32 / 1.01 * ew as f32);
    }

    // Displacement
    for (i, (mut transform, _)) in query.iter_mut().enumerate() {
        let dx = velocities[i].x;
        let dy = velocities[i].y;
        let distance = (dx * dx + dy * dy).sqrt();
        if distance > 0.1 {
            velocities[i].x += dx / distance * cooling_factor.min(distance);
            velocities[i].y += dy / distance * cooling_factor.min(distance);

            // Do some kinda division here to make sure that the actual translation doesn't go
            // outside the bounds, not the velocity
            velocities[i].x = velocities[i].x.max(-area / 2.0).min(area / 2.0);
            velocities[i].y = velocities[i].y.max(-area / 2.0).min(area / 2.0);

            transform.translation += velocities[i].with_z(0.0);
        }
    }
}

fn handle_node_physics_old(
    mut query: Query<(&mut Transform, &NodeController)>,
    time: Res<Time>,
    napkin: Res<NapkinSettings>,
) {
    let k: f32 = 1.0;
    let k_sqr = k * k;
    let nodes = query
        .iter_mut()
        .map(|(transform, node_controller)| (transform.translation, node_controller))
        .collect::<Vec<_>>();
    let area = (nodes.len() as f32).sqrt() * (6.0 * 6.0);
    let temp = k_sqr / area;
    let mut kdtree = KdTree::new(2);
    let mut velocities = vec![Vec3::ZERO; nodes.len()];
    let delta_time = time.delta_seconds();
    for i in 0..nodes.len() {
        let node_position = [nodes[i].0.x, nodes[i].0.y];
        kdtree.add(node_position, i).unwrap();
    }

    for i in 0..nodes.len() {
        let mut repulsion_force = Vec2::ZERO;
        let mut attraction_force = Vec2::ZERO;

        let node_position = [nodes[i].0.x, nodes[i].0.y];
        let node_size = 6.0 * 3.0;

        // Repulsion and attraction for nodes
        for j in 0..nodes.len() {
            if j != i {
                let connected = napkin.edges.iter().any(|edge|
                    (edge.source == nodes[i].1.id && edge.target == nodes[j].1.id) ||
                    (edge.source == nodes[i].1.id && edge.source == nodes[j].1.id)
                );
                let edge_weight = if connected { 0.8 } else { 1.0 };
                let distance = nodes[i].0 - nodes[j].0;
                let distance_sqr = distance.length_squared();
                let weighted_distance = (k * k * node_size * node_size) / (distance_sqr * edge_weight);
                if connected {
                    info!("id: {}, direction: {}, distance: {}, w1: {}, w2: {}, w: {}", i, distance.xy().normalize(), distance.length(), (k * k * node_size * node_size), (distance_sqr * edge_weight), weighted_distance);
                }
                repulsion_force += distance.xy().normalize() * weighted_distance;
            }
        }

        // Attraction force
        let mut center = Vec2::ZERO;
        let mut total_weight = 0.0;
        // if let Some(neighbors) = kdtree.within(&node_position, 1000.0, &kdtree::distance::squared_euclidean).ok() {
        //     for neighbor in neighbors {
        //         if *neighbor.1 != i {
        //             let j = *neighbor.1;
        //             let connected = napkin.edges.iter().any(|edge|
        //                 (edge.source == nodes[i].1.id && edge.target == nodes[j].1.id) ||
        //                 (edge.source == nodes[i].1.id && edge.source == nodes[j].1.id)
        //             );
        //             let edge_weight = if connected { 2.0 } else { 1.0 };
        //             total_weight += edge_weight;
        //             center += (nodes[j].0 - nodes[i].0).xy() * edge_weight;
        //         }
        //     }
        // }

        if total_weight > 0.0 {
            center /= total_weight;
            let distance = center.length();
            let weighted_distance = (k * k * node_size) / (distance * distance);
            attraction_force += center.normalize() * weighted_distance;
        }

        // Gravity
        let gravity_force = -nodes[i].0.xy().normalize() * 0.6;

        // Final velocity, including jittering
        let mut velocity = repulsion_force + attraction_force + gravity_force;
        let jitter = Vec2::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5) * 0.1;
        velocity += jitter * 0.1;
        let speed = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
        if speed > area {
            let ratio = area / speed;
            velocity.x *= ratio;
            velocity.y *= ratio;
        }
        velocities[i] = Vec3::new(velocity.x, velocity.y, 0.0);
    }

    for (i, (mut transform, _)) in query.iter_mut().enumerate() {
        transform.translation += velocities[i].with_z(0.0);
    }


    // for i in 0..nodes.len() {
    //     let node_position = nodes[i].0;
    //     // Attraction to center
    //     let center_direction = center - node_position;
    //     let center_distance = center_direction.length();
    //     if center_distance > 0.0 {
    //         let center_force_magnitude = center_distance * 0.4;
    //         velocities[i] += center_direction.normalize() * center_force_magnitude * delta_time;
    //     }

    //     // Repulsion between nodes
    //     for j in 0..nodes.len() {
    //         if i != j {
    //             let direction = node_position - nodes[j].0;
    //             let distance = direction.length();
    //             let connected = napkin.edges.iter().any(|edge|
    //                 (edge.source == nodes[i].1.id && edge.target == nodes[j].1.id) ||
    //                 (edge.source == nodes[i].1.id && edge.source == nodes[j].1.id)
    //             );
    //             let repulsion_factor = if connected { 500.0 } else { 1000.0 };
    //             // Repulsive force inverse to distance
    //             let force_magnitude = repulsion_factor / distance.max(50.0);
    //             velocities[i] += direction.normalize() * force_magnitude * delta_time;    
    //         }
    //     }
    // }

    // Update positions and apply damping to simulate friction
    // let damping_factor = 1.2;
    // for (i, (mut transform, _)) in query.iter_mut().enumerate() {
    //     velocities[i] *= damping_factor;
    //     if velocities[i].length() < 0.05 {
    //         velocities[i] = Vec3::ZERO;
    //     }
    //     transform.translation += velocities[i].with_z(0.0);
    // }
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

pub fn update_controller(
    mut napkin: ResMut<NapkinSettings>,
    mut nodes: Query<(&mut NodeController, &mut Mesh2dHandle, &mut Handle<ColorMaterial>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // for (mut controller, mut mesh, color_material) in nodes.iter_mut() {
    //     let material = materials.get_mut(color_material.into_inner()).unwrap();
    //     let node = napkin.nodes.iter().find(|n| n.id == controller.id).unwrap();

    //     if controller.project == node.project {
    //         controller.project = node.project.clone();
    //     }

    //     if napkin.selected_project.is_none() || napkin.selected_project == Some(node.project.clone()) {
    //         mesh.0 = meshes.add(Circle::new(6.0)).into();
    //         material.color = Color::WHITE;
    //     } else {
    //         mesh.0 = meshes.add(Rectangle::new(10.0, 10.0)).into();
    //         material.color = Color::linear_rgb(0.4, 0.2, 0.2);
    //     }
    // }
}
