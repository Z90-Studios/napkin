use std::fmt::{self, Debug};

use bevy::{
    core_pipeline::bloom::BloomSettings,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{
    egui::{self, Color32, RichText, Vec2, Vec2b},
    EguiContexts, EguiPlugin, EguiSettings,
};
use bevy_http_client::prelude::*;
use bevy_infinite_grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};
use bevy_rapier3d::prelude::*;
use egui_plot::{Line, Plot, PlotPoints};
use serde::{Deserialize, Serialize};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

mod plugins;

use plugins::{
    camera_controller,
    crosshair_controller::CrosshairControllerPlugin,
    edge_controller::{self, EdgeControllerPlugin},
    edge_metadata_controller::EdgeMetadataControllerPlugin,
    node_controller::{self, NodeControllerPlugin},
    node_metadata_controller::NodeMetadataControllerPlugin,
    project_controller::ProjectControllerPlugin,
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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct NapkinProject {
    id: String,
    scope: String,
    name: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct NapkinNode {
    project: String,
    id: String,
}

impl fmt::Display for NapkinNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NapkinNode {{ project: {}, id: {} }}",
            self.project, self.id
        )
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
        write!(
            f,
            "NapkinEdge {{ project: {}, id: {}, source: {}, target: {} }}",
            self.project, self.id, self.source, self.target
        )
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct NapkinNodeMetadata {
    owner_id: String,
    name: String,
    value: serde_json::Value,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct NapkinEdgeMetadata {
    owner_id: String,
    name: String,
    value: serde_json::Value,
}

// TODO: Consider implementing actual types for some of these
#[derive(Resource)]
struct NapkinSettings {
    server_url: String,
    is_connected: bool,
    selected_project: Option<String>, // Project UUID
    napkin_crosshair: NapkinCrosshair,
    hovered_nodes: Option<Vec<NapkinNode>>,
    hovered_edges: Option<Vec<NapkinEdge>>,
    selected_nodes: Option<Vec<NapkinNode>>, // Multiple Selection Shift+Click
    selected_edges: Option<Vec<String>>,     // Same, but separated for fun
    nodes: Vec<NapkinNode>,
    node_metadata: Vec<NapkinNodeMetadata>,
    edges: Vec<NapkinEdge>,
    edge_metadata: Vec<NapkinEdgeMetadata>,
    projects: Vec<NapkinProject>,
    project_search_string: String,
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
            node_metadata: Vec::new(),
            edges: Vec::new(),
            edge_metadata: Vec::new(),
            projects: Vec::new(),
            project_search_string: String::new(),
        }
    }
}

#[derive(Default, Resource)]
struct AtlasDiagnostics {
    fps_log: Vec<f64>,
}

fn main() {
    App::new()
        .init_resource::<AtlasDiagnostics>()
        .init_resource::<OccupiedScreenSpace>()
        .init_resource::<NapkinSettings>()
        .insert_resource(Msaa::Sample8) // TODO: Implement a --performance-mode flag or other setting to disable this and other performance tweaks
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .register_request_type::<Vec<NapkinProject>>()
        .register_request_type::<Vec<NapkinNode>>()
        .register_request_type::<Vec<NapkinEdge>>()
        .register_request_type::<Vec<NapkinNodeMetadata>>()
        .register_request_type::<Vec<NapkinEdgeMetadata>>()
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
            ProjectControllerPlugin,
            NodeControllerPlugin,
            EdgeControllerPlugin,
            NodeMetadataControllerPlugin,
            EdgeMetadataControllerPlugin,
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
    mut atlas_diagnostics: ResMut<AtlasDiagnostics>,
    mut napkin: ResMut<NapkinSettings>,
    windows: Query<&mut Window>,
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut project_request: EventWriter<TypedRequest<Vec<NapkinProject>>>,
    mut node_request: EventWriter<TypedRequest<Vec<NapkinNode>>>,
    mut edge_request: EventWriter<TypedRequest<Vec<NapkinEdge>>>,
    mut node_metadata_request: EventWriter<TypedRequest<Vec<NapkinNodeMetadata>>>,
    mut edge_metadata_request: EventWriter<TypedRequest<Vec<NapkinEdgeMetadata>>>,
    // mut egui_settings: ResMut<EguiSettings>,
) {
    let ctx = contexts.ctx_mut();

    let atlas_panel_frame = egui::Frame {
        fill: egui::Color32::from_black_alpha((255. * 0.9) as u8),
        inner_margin: egui::Margin {
            left: 4.,
            right: 4.,
            top: 4.,
            bottom: 4.,
        },
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
                        "TODO: Implement node/edge/metadata list, search, filter here",
                    ));

                    if ui
                        .add(egui::Button::new(egui::RichText::new("Refresh")).wrap(true))
                        .clicked()
                    {
                        project_request.send(
                            HttpClient::new()
                                .get(format!("{}/project", napkin.server_url))
                                .with_type::<Vec<NapkinProject>>(),
                        );
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
                        node_metadata_request.send(
                            HttpClient::new()
                                .get(format!("{}/node/metadata", napkin.server_url))
                                .with_type::<Vec<NapkinNodeMetadata>>(),
                        );
                        edge_metadata_request.send(
                            HttpClient::new()
                                .get(format!("{}/edge/metadata", napkin.server_url))
                                .with_type::<Vec<NapkinEdgeMetadata>>(),
                        );
                    };
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
            ui.group(|ui| {
                ui.label(
                    if let Some(project) = &napkin
                        .projects
                        .iter()
                        .find(|&project| Some(&project.id) == napkin.selected_project.as_ref())
                    {
                        format!("Current Project: @{}/{}", &project.scope, &project.name)
                    } else {
                        "Current Project: None".to_string()
                    },
                );
                ui.horizontal_wrapped(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(&mut napkin.project_search_string);
                });
                let project_search_string = napkin.project_search_string.clone();
                let filtered_projects = napkin
                    .projects
                    .iter()
                    .filter(|project| {
                        format!("@{}/{}", project.scope, project.name)
                            .contains(&project_search_string)
                    })
                    .cloned() // Clone the filtered projects to avoid borrowing issue
                    .collect::<Vec<_>>();
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for project in filtered_projects {
                            if ui
                                .button(RichText::new(format!(
                                    "@{}/{}",
                                    project.scope, project.name
                                )))
                                .clicked()
                            {
                                napkin.selected_project = Some(project.id.clone());
                            }
                            // ui.label(format!("@{}/{}", project.scope, project.name));
                        }
                    })
                });
            });
        })
        .response
        .rect
        .height();
    occupied_screen_space.bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .frame(atlas_panel_frame)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(
                "TODO: Implement node/edge/metadata viewer/editor here",
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
            // ui.horizontal(|ui| {
            //     ui.label(egui::RichText::new("Connection Status:").strong());
            //     if napkin.is_connected {
            //         ui.colored_label(egui::Color32::GREEN, "Connected");
            //     } else {
            //         ui.colored_label(egui::Color32::RED, "Disconnected");
            //     }
            // });
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.vertical(|ui| {
                    // FPS Graph
                    let mut fps_data = Vec::new();
                    for (index, value) in atlas_diagnostics.fps_log.iter().enumerate() {
                        fps_data.push([index as f64 * 0.01, *value as f64]);
                    }
                    let fps_line = Line::new(fps_data).color({
                        if let Some(last_fps) = atlas_diagnostics.fps_log.last() {
                            if *last_fps > (60. * 0.9) {
                                egui::Color32::GREEN
                            } else if *last_fps > (60. * 0.7) {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::RED
                            }
                        } else {
                            egui::Color32::GRAY // Default color if no fps data is available
                        }
                    });

                    // RAM Utilization Graph
                    let mut ram_data = Vec::new();
                    for (index, value) in atlas_diagnostics.ram_log.iter().enumerate() {
                        ram_data.push([index as f64 * 0.01, *value as f64]);
                    }
                    let ram_line = Line::new(ram_data).color({
                        if let Some(last_ram) = atlas_diagnostics.ram_log.last() {
                            if *last_ram < 80.0 {
                                egui::Color32::GREEN
                            } else if *last_ram < 90.0 {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::RED
                            }
                        } else {
                            egui::Color32::GRAY // Default color if no ram data is available
                        }
                    });

                    // Display FPS and RAM values
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if let Some(fps_value) = diagnostics
                                .get(&FrameTimeDiagnosticsPlugin::FPS)
                                .and_then(|fps| fps.smoothed())
                            {
                                atlas_diagnostics.fps_log.push(fps_value);
                                ui.label(format!("{:.2} FPS", fps_value));
                            } else {
                                ui.label("FPS: N/A");
                            }

                            if let Some(ram_value) = diagnostics
                                .get(&FrameTimeDiagnosticsPlugin::RAM)
                                .and_then(|ram| ram.smoothed())
                            {
                                atlas_diagnostics.ram_log.push(ram_value);
                                ui.label(format!("{:.2} MB", ram_value));
                            } else {
                                ui.label("RAM: N/A");
                            }
                        });
                    });

                    // Plotting both FPS and RAM utilization
                    Plot::new("performance_plot")
                        .view_aspect(3.0)
                        .show_axes(Vec2b { x: false, y: true })
                        .show_x(false)
                        .label_formatter(|name, value| {
                            format!("{}: {:.2}", name, value.y)
                        })
                        .show(ui, |plot_ui| {
                            plot_ui.line(fps_line);
                            plot_ui.line(ram_line);
                        });

                    ui.allocate_space(ui.available_size());
                });
            });
        });
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
