use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{
    egui::{self, Color32},
    EguiContexts,
};
use bevy_http_client::prelude::TypedResponse;
use bevy_rapier3d::{
    dynamics::{GravityScale, ImpulseJoint, RapierRigidBodyHandle, RigidBody, SphericalJoint, SphericalJointBuilder},
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
        app.add_systems(Update, (run_edge_controller, edge_tooltip));
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

        // Cast a ray for nodes, so we know not to trigger edge clicking when the node is hit
        let node_hit = rapier_context.cast_ray(
            ray.origin,
            ray.direction.into(),
            f32::MAX,
            true,
            QueryFilter::new().groups(CollisionGroups::new(Group::ALL, Group::GROUP_13)),
        );

        // Cast the ray
        let hit = rapier_context.cast_ray(
            ray.origin,
            ray.direction.into(),
            f32::MAX,
            true,
            QueryFilter::new().groups(CollisionGroups::new(Group::ALL, Group::GROUP_14)),
        );

        if let None = node_hit {
            if let Some((entity, _toi)) = hit {
                commands.entity(entity).insert(HoveredEdge);
            }
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

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Generates a pleasing random color based on a hash of the given value.
/// Uses HSL color space to ensure the color is pleasing.
fn generate_pleasing_color<T: Hash>(value: &T) -> Color32 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    let hash = hasher.finish();

    // Convert hash to a float between 0 and 1 for hue selection in HSL
    let hue = (hash % 360) as f32; // Hue value between 0 and 360
    let saturation = 0.75; // High saturation for vivid colors
    let lightness = 0.5; // Normal lightness for balanced brightness

    // Convert HSL to RGB, then to Color32
    let rgb = hsl_to_rgb(hue, saturation, lightness);
    Color32::from_rgb(rgb.0, rgb.1, rgb.2)
}

/// Converts an HSL color value to RGB.
/// Assumes h is in [0, 360], s in [0, 1], and l in [0, 1].
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match h as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

pub fn edge_tooltip(
    mut napkin: ResMut<NapkinSettings>,
    windows: Query<&mut Window>,
    mut contexts: EguiContexts,
) {
    let ctx = contexts.ctx_mut();

    if let (Some(mouse_position), window_height) = (
        windows.single().cursor_position(),
        windows.single().height(),
    ) {
        let edge_tooltip_frame = egui::Frame {
            fill: egui::Color32::from_black_alpha((255. * 0.99) as u8),
            stroke: egui::Stroke::NONE,
            ..egui::Frame::none()
        };
        let offset_position = [
            mouse_position.x + 6.,
            (mouse_position.y - 6.) - window_height,
        ];
        // TODO: Make it so you can lock a tooltip and click on things
        if let Some(hovered_edges) = &napkin.hovered_edges {
            if !hovered_edges.is_empty() {
                egui::Window::new("Edge Details")
                    .id(egui::Id::new("edge_tooltip"))
                    .title_bar(false)
                    .resizable(false)
                    .frame(edge_tooltip_frame)
                    .anchor(egui::Align2::LEFT_BOTTOM, offset_position)
                    .show(ctx, |ui| {
                        for edge in hovered_edges {
                            let _project = &napkin
                                .projects
                                .iter()
                                .find(|&project| project.id == edge.project);
                            if let Some(project) = _project {
                                ui.label(format!("@{}/{}", project.scope, project.name))
                                    .highlight();
                            }
                            ui.label(format!("Edge ID: {}", edge.id));
                            ui.label("Metadata");
                            // let metadata_color = Color32::from_rgb(180, 180, 220);
                            for metadata in napkin
                                .edge_metadata
                                .iter()
                                .filter(|&edge_metadata| edge_metadata.owner_id == edge.id)
                            {
                                let metadata_color = generate_pleasing_color(&metadata.name);
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

pub fn edge_spawner(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut napkin: ResMut<NapkinSettings>,
    existing_nodes: Query<(Entity, &mut NodeController)>,
    existing_edges: Query<&mut EdgeController>,
    mut ev_response: EventReader<TypedResponse<Vec<NapkinEdge>>>,
) {
    for response in ev_response.read() {
        info!("Received edges from server");
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
                if existing_node.1.id == edge.source {
                    source_node = Some(existing_node);
                } else if existing_node.1.id == edge.target {
                    target_node = Some(existing_node);
                }
            }
            if let Some((source_entity, source)) = source_node {
                if let Some((target_entity, target)) = target_node {
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
                            RigidBody::Dynamic,
                            GravityScale(0.0),
                            Collider::capsule(source.position, target.position, 0.03),
                            // ImpulseJoint::new(
                            //     source_entity,
                            //     SphericalJointBuilder::new()
                            //         .local_anchor1(Vec3::ZERO)
                            //         .local_anchor2(Vec3::ZERO),
                            // ),
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
