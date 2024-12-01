use bevy::sprite::Material2d;
use bevy::{prelude::*, window::CursorGrabMode};
use bevy::window::PrimaryWindow;
use bevy_egui::{
    egui,
    egui::CursorIcon,
    EguiContexts,
    EguiPlugin,
};
use bevy_rapier2d::prelude::*;
use bevy_http_client::prelude::*;

mod types;
mod plugins;

use plugins::camera_controller::{CameraController, CameraControllerPlugin};
use types::napkin_types::*;

#[derive(Default, Resource)]
pub struct OccupiedScreenSpace {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

#[derive(Component)]
pub struct HoveredNode;

#[derive(Component)]
pub struct NodeController {
    project: String,
    id: String,
}

pub struct NodeControllerPlugin;

impl Plugin for NodeControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                run_node_controller,
                // node_tooltip,
                cast_ray,
                // node_spawner,
                // node_destroyer,
                // handle_node_click,
                // handle_node_physics,
            ),
        );
    }
}

impl Default for NodeController {
    fn default() -> Self {
        Self {
            project: "Unknown".to_string(),
            id: "1234".to_string(),
        }
    }
}

#[derive(Resource)]
pub struct NapkinSettings {
    server_url: String,
    is_connected: bool,
    selected_project: Option<String>, // Project UUID
    // napkin_crosshair: NapkinCrosshair,
    hovered_nodes: Option<Vec<NapkinNode>>,
    // hovered_edges: Option<Vec<NapkinEdge>>,
    selected_nodes: Option<Vec<NapkinNode>>, // Multiple Selection Shift+Click
    // selected_edges: Option<Vec<String>>,     // Same, but separated for fun
    nodes: Vec<NapkinNode>,
    // node_metadata: Vec<NapkinNodeMetadata>,
    // edges: Vec<NapkinEdge>,
    // edge_metadata: Vec<NapkinEdgeMetadata>,
    // projects: Vec<NapkinProject>,
    project_search_string: String,
}

impl Default for NapkinSettings {
    fn default() -> Self {
        Self {
            server_url: "http://127.0.0.1:28527".to_string(),
            is_connected: false,
            selected_project: None,
            // napkin_crosshair: NapkinCrosshair::default(),
            hovered_nodes: None,
            // hovered_edges: None,
            selected_nodes: None,
            // selected_edges: None,
            nodes: Vec::new(),
            // node_metadata: Vec::new(),
            // edges: Vec::new(),
            // edge_metadata: Vec::new(),
            // projects: Vec::new(),
            project_search_string: String::new(),
        }
    }
}

fn main() {
    App::new()
        .init_resource::<OccupiedScreenSpace>()
        .init_resource::<NapkinSettings>()
        .register_request_type::<Vec<NapkinProject>>()
        .register_request_type::<Vec<NapkinNode>>()
        .register_request_type::<Vec<NapkinEdge>>()
        .register_request_type::<Vec<NapkinNodeMetadata>>()
        .register_request_type::<Vec<NapkinEdgeMetadata>>()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins((
            HttpClientPlugin,
            NodeControllerPlugin,
            CameraControllerPlugin,
        ))
        // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
        // or after the `EguiSet::BeginPass` system (which belongs to the `CoreSet::PreUpdate` set).
        .add_systems(Startup, (setup_camera, test_node))
        .add_systems(Update, setup_ui)
        .run();
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        CameraController::default(),
    ));
}

pub fn run_node_controller(
    mut napkin: ResMut<NapkinSettings>,
    _time: Res<Time>,
    mut node_set: ParamSet<(
        Query<(&GlobalTransform, &mut Transform, &mut NodeController), Without<Camera>>,
        Query<(&mut HoveredNode, &NodeController), Without<Camera>>,
    )>,
) {
    // let target = camera.single();
    // for (global_transform, mut node_pos, mut node_controller) in node_set.p0().iter_mut() {
    //     let start = node_pos.translation;
    //     let forward = start - target.translation;
    //     node_pos.look_at(start + forward, Vec2::Y);
    //     node_controller.position = global_transform.translation();
    // }

    let mut new_selected_nodes: Vec<NapkinNode> = Vec::new();
    for (_, node_controller) in node_set.p1().iter_mut() {
        new_selected_nodes.push(NapkinNode {
            project: node_controller.project.clone(),
            id: node_controller.id.clone(),
        });
    }
    napkin.hovered_nodes = Some(new_selected_nodes);
}

pub fn cast_ray(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    rapier_context: Res<RapierContext>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut nodes: Query<(Entity, &mut Handle<ColorMaterial>), With<NodeController>>,
    mut contexts: EguiContexts,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ctx = contexts.ctx_mut();
    let window = windows.single();

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

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
            QueryFilter::new().groups(CollisionGroups::new(Group::ALL, Group::GROUP_13)),
            |e| {
                // Callback called on each collider hit by the ray.
                entity = Some(e);
                commands.entity(e).insert(HoveredNode);
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
                    material.color = Color::linear_rgb(1., 0., 0.);
                }
            } else {
                commands.entity(n_entity).remove::<HoveredNode>();
                material.color = Color::WHITE;
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

fn test_node(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        bevy::sprite::MaterialMesh2dBundle {
            mesh: meshes.add(Circle::new(10.0)).into(),
            transform: Transform::default(),
            material: materials.add(ColorMaterial::from(Color::WHITE)),
            ..default()
        },
        NodeController::default(),
        RigidBody::Dynamic,
        Collider::ball(10.0),
        GravityScale(0.0),
        // Assign the node to collision group 13 and allow it to collide with group 4
        CollisionGroups::new(Group::GROUP_13, Group::GROUP_4),
        // Assign the node to solver group 3 and allow interaction with solver group 11
        SolverGroups::new(Group::GROUP_13, Group::GROUP_4),
    ));
}

fn setup_ui(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
) {
    let ctx = contexts.ctx_mut();

    occupied_screen_space.left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Atlas 2d");
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
}
