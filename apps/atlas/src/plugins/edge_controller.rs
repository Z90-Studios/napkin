use bevy::{
    prelude::*, sprite::Mesh2dHandle, window::{CursorGrabMode, PrimaryWindow}
};
use bevy_egui::{
    egui::{self, Color32, CursorIcon},
    EguiContexts,
};
use bevy_http_client::{prelude::{TypedRequest, TypedResponse}, HttpClient};
use bevy_rapier2d::{
    dynamics::{
        GravityScale, ImpulseJoint, RapierRigidBodyHandle, RigidBody, RopeJointBuilder
    },
    geometry::{Collider, CollisionGroups, Group, SolverGroups},
    pipeline::QueryFilter,
    plugin::RapierContext,
};
use std::{f32::consts::PI, fmt};

use crate::{NapkinEdge, NapkinEdits, NapkinSettings, OccupiedScreenSpace};

use super::{camera_controller::{self, CameraController}, node_controller::NodeController};

pub struct EdgeControllerPlugin;

impl Plugin for EdgeControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
                run_edge_controller,
                edge_spawner,
                edge_destroyer,
                handle_edge_physics,
                handle_click,
                cast_ray,
                save_edge,
                apply_response,
                update_controller,
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
    napkin: ResMut<NapkinSettings>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    edges: Query<(Entity, &mut EdgeController, &mut Handle<ColorMaterial>, Option<&HoveredEdge>)>,
    mut contexts: EguiContexts,
) {
    let ctx = contexts.ctx_mut();
    
    let selected_node = napkin.selected_edge.clone().unwrap_or("".to_string());
    for (_, edge_controller, color_material, hovered) in &mut edges.iter() {
        let selected = selected_node == edge_controller.id;
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

pub fn handle_click(
    hovered_edges: Query<(&HoveredEdge, &EdgeController)>,
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
        for (_, edge) in &hovered_edges {
            napkin.selected_edge = Some(edge.id.clone());
            return;
        }
    }
}

fn cast_ray(
    mut commands: Commands,
    napkin: Res<NapkinSettings>,
    occupied_screen_space: Res<OccupiedScreenSpace>,
    windows: Query<&Window, With<PrimaryWindow>>,
    rapier_context: Res<RapierContext>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut camera_controller_query: Query<&mut CameraController>,
    hovered_edges: Query<Entity, With<HoveredEdge>>,
) {
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
            |_| {
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
    
                    true
                },
            );
        }


        if let Some(e) = entity {
            camera_controller.enabled = false;
            for entity in hovered_edges.iter() {
                if entity != e {
                    commands.entity(entity).remove::<HoveredEdge>();
                }
            }
            commands.entity(e).insert(HoveredEdge);
        } else {
            for entity in hovered_edges.iter() {
                commands.entity(entity).remove::<HoveredEdge>();
            }
            return;
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

pub fn save_edge(
    mut napkin: ResMut<NapkinSettings>,
    mut edits: ResMut<NapkinEdits>,
    mut edge_request: EventWriter<TypedRequest<NapkinEdge>>,
) {
    if edits.save_edge {
        let mut http_client = HttpClient::new();
        if edits.edge.id.is_empty() {
            http_client = http_client.post(format!("{}/edge", napkin.server_url));
        } else {
            http_client = http_client.put(format!("{}/edge/{}", napkin.server_url, edits.edge.id));
        }
        http_client = http_client.json(&edits.edge);
        edge_request.send(
            http_client.with_type::<NapkinEdge>(),
        );
        edits.save_edge = false;
    }
}

pub fn apply_response(
    mut napkin: ResMut<NapkinSettings>,
    mut edits: ResMut<NapkinEdits>,
    mut edge_response: EventReader<TypedResponse<NapkinEdge>>,
) {
    for response in edge_response.read() {
        info!("Received updated edge from server");
        let edge = NapkinEdge {
            id: response.id.clone(),
            project: response.project.clone(),
            source: response.source.clone(),
            target: response.target.clone(),
        };
        edits.edge = edge.clone();
        let mut exists = false;
        for e in napkin.edges.iter_mut() {
            if e.id == edge.id {
                exists = true;
                *e = edge.clone();
            }
        }
        if exists == false {
            napkin.edges.push(edge.clone());
        }
    }
}

pub fn update_controller(
    napkin: Res<NapkinSettings>,
    mut edges: Query<&mut EdgeController>,
) {
    for mut controller in edges.iter_mut() {
        let edge = napkin.edges.iter().find(|e| e.id == controller.id).unwrap();

        /*if controller.project == edge.project
            && controller.source == edge.source
            && controller.target == edge.target
        {
            return;
        } else {*/
            controller.project = edge.project.clone();
            controller.source = edge.source.clone();
            controller.target = edge.target.clone();
        //}
    }
}
