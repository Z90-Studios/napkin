use std::fmt::{self, Debug};

use bevy::{
    core_pipeline::bloom::BloomSettings,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{
    egui::{self, Color32},
    EguiContexts, EguiPlugin, EguiSettings,
};
use bevy_http_client::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

mod plugins;

use plugins::{
    camera_controller, crosshair_controller::CrosshairControllerPlugin, edge_controller::{self, EdgeControllerPlugin}, node_controller::{self, NodeControllerPlugin}
};

#[derive(Default, Resource)]
struct OccupiedScreenSpace {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

#[derive(Default)]
enum NapkinCrosshairSelectionTypes {
    #[default]
    Node,
    Edge,
}

#[derive(Default)]
struct NapkinCrosshair {
    selected_type: NapkinCrosshairSelectionTypes,
    selected_id: Option<String>,

}
#[derive(Default)]
struct NapkinProject {
    id: String,
    name: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct NapkinNode {
    project: String,
    id: String,
}

impl fmt::Display for NapkinNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NapkinNode {{ project: {}, id: {} }}", self.project, self.id)
    }
}


#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct NapkinEdge {
    project: String,
    id: String,
    source: String,
    target: String,
}

impl fmt::Display for NapkinEdge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NapkinEdge {{ project: {}, id: {}, source: {}, target: {} }}", self.project, self.id, self.source, self.target)
    }
}

// TODO: Consider implementing actual types for some of these
#[derive(Resource)]
struct NapkinSettings {
    server_url: String,
    is_connected: bool,
    selected_project: Option<String>,   // Project UUID
    napkin_crosshair: NapkinCrosshair,
    hovered_nodes: Option<Vec<NapkinNode>>, 
    hovered_edges: Option<Vec<NapkinEdge>>,
    selected_nodes: Option<Vec<NapkinNode>>, // Multiple Selection Shift+Click
    selected_edges: Option<Vec<String>>, // Same, but separated for fun
    nodes: Vec<NapkinNode>,
    edges: Vec<NapkinEdge>,
    projects: Vec<NapkinProject>,
}

impl Default for NapkinSettings {
    fn default() -> Self {
        Self {
            server_url: "http://127.0.0.1:28527".to_string(),
            is_connected: false,
            selected_project: None,
            napkin_crosshair: NapkinCrosshair::default(),
            hovered_nodes: None,
            hovered_edges: None,
            selected_nodes: None,
            selected_edges: None,
            nodes: Vec::new(),
            edges: Vec::new(),
            projects: Vec::new(),
        }
    }
}

fn main() {
    App::new()
        .init_resource::<OccupiedScreenSpace>()
        .init_resource::<NapkinSettings>()
        .insert_resource(Msaa::Sample8) // TODO: Implement a --performance-mode flag or other setting to disable this and other performance tweaks
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .register_request_type::<Vec<NapkinNode>>()
        .register_request_type::<Vec<NapkinEdge>>()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin,
            HttpClientPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            // RapierDebugRenderPlugin::default(),
            LookTransformPlugin,
            OrbitCameraPlugin::new(true),
            // CameraControllerPlugin,
            // InfiniteGridPlugin,
            NodeControllerPlugin,
            EdgeControllerPlugin,
            CrosshairControllerPlugin,
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, (configure_visuals_system, setup_system))
        .add_systems(
            Update,
            (
                setup_ui,
                camera_controller::atlas_orbit_camera_input_map,
                edge_controller::cast_ray,
                edge_controller::edge_spawner,
            ),
        )
        .run()
}

fn configure_visuals_system(mut contexts: EguiContexts) {
    let atlas_visuals = egui::Visuals {
        window_rounding: 0.0.into(),
        window_fill: Color32::from_rgba_premultiplied(46, 64, 83, 40),
        window_stroke: contexts
            .ctx_mut()
            .style()
            .visuals
            .widgets
            .noninteractive
            .fg_stroke,
        ..Default::default()
    };
    contexts.ctx_mut().set_visuals(atlas_visuals);
}

fn setup_ui(
    mut napkin: ResMut<NapkinSettings>,
    diagnostics: Res<DiagnosticsStore>,
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut node_request: EventWriter<TypedRequest<Vec<NapkinNode>>>,
    mut edge_request: EventWriter<TypedRequest<Vec<NapkinEdge>>>,
    mut egui_settings: ResMut<EguiSettings>,
) {
    let ctx = contexts.ctx_mut();
    egui_settings.scale_factor = 1.25;

    let atlas_panel_frame = egui::Frame {
        fill: egui::Color32::from_black_alpha((255. * 0.9) as u8),
        ..egui::Frame::none()
    };

    occupied_screen_space.left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .frame(atlas_panel_frame)
        .show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.label(egui::RichText::new(
                        "TODO: Implement node/edge/metadata list, search, filter here"
                    ));
                    if napkin.is_connected {
                        ui.add(egui::Button::new(egui::RichText::new("Connected ✓")).wrap(true));
                    } else {
                        if ui
                            .add(
                                egui::Button::new(egui::RichText::new("Not Connected ❌"))
                                    .wrap(true),
                            )
                            .clicked()
                        {
                            node_request.send(
                                HttpClient::new()
                                    .get(format!("{}/node", napkin.server_url))
                                    .with_type::<Vec<NapkinNode>>(),
                            );
                            edge_request.send(
                                HttpClient::new()
                                    .get(format!("{}/edge", napkin.server_url))
                                    .with_type::<Vec<NapkinEdge>>(),
                            );
                        };
                    }
                },
            );
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
    occupied_screen_space.top = egui::TopBottomPanel::top("top_panel")
        .resizable(true)
        .frame(atlas_panel_frame)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(
                "TODO: Implement project list, search, filter here"
            ));
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();
    occupied_screen_space.bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .frame(atlas_panel_frame)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(
                "TODO: Implement node/edge/metadata viewer/editor here"
            ));
            if let Some(hovered_nodes) = &napkin.hovered_nodes {
                for node in hovered_nodes {
                    ui.label(egui::RichText::new(node.to_string()));
                }
            }
            if let Some(hovered_edges) = &napkin.hovered_edges {
                for edge in hovered_edges {
                    ui.label(egui::RichText::new(edge.to_string()));
                }
            }
            if let Some(selected_nodes) = &napkin.selected_nodes {
                for node in selected_nodes {
                    ui.label(egui::RichText::new(format!("Node selected! ({})", node.id)));
                }
            }
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();
    occupied_screen_space.right = egui::SidePanel::right("right_panel")
        .resizable(true)
        .frame(atlas_panel_frame)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(
                "TODO: Implement connection status, stats, here"
            ));
            // ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
            if let Some(value) = diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FPS)
                .and_then(|fps| fps.smoothed())
            {
                ui.add(egui::Label::new(format!("{:.2} FPS", value)));
            } else {
                ui.add(egui::Label::new("FPS: N/A"));
            }
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
}

fn setup_system(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_3)),
        ..Default::default()
    });

    commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    hdr: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            BloomSettings::NATURAL,
        ))
        .insert(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            Vec3::new(-2.0, 5.0, 5.0),
            Vec3::new(0., 0., 0.),
            Vec3::Y,
        ));
    commands.spawn(InfiniteGridBundle {
        settings: InfiniteGridSettings {
            x_axis_color: Color::rgb(0.8, 0.8, 0.8),
            z_axis_color: Color::rgb(0., 1., 0.),
            scale: 10.,
            ..Default::default()
        },
        ..Default::default()
    });
}
