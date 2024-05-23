use std::fmt::{self, Debug};

use bevy::{
    core_pipeline::bloom::BloomSettings,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{
    egui::{self, Color32, RichText, Vec2b},
    EguiContexts, EguiPlugin
};
use bevy_http_client::prelude::*;
use bevy_rapier3d::prelude::*;
use egui_plot::{Line, Plot};
use serde::{Deserialize, Serialize};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

mod plugins;

use plugins::{
    camera_controller,
    crosshair_controller::CrosshairControllerPlugin,
    debug_controller::DebugControllerPlugin,
    edge_controller::EdgeControllerPlugin,
    edge_metadata_controller::EdgeMetadataControllerPlugin,
    napkin_controller::NapkinPlugin,
    node_controller::NodeControllerPlugin,
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
struct NapkinCrosshair {
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
    fps_log: Vec<(f64, f64)>, // Tuple (uptime_at, fps)
    uptime: f64,
    debug_mode: bool,
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
            NapkinPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            LookTransformPlugin,
            OrbitCameraPlugin::new(true),
            // CameraControllerPlugin,
            ProjectControllerPlugin,
            NodeControllerPlugin,
            EdgeControllerPlugin,
            NodeMetadataControllerPlugin,
            EdgeMetadataControllerPlugin,
            CrosshairControllerPlugin,
            FrameTimeDiagnosticsPlugin,
        ))
        .add_plugins((DebugControllerPlugin,))
        .add_systems(Startup, (configure_visuals_system, setup_system))
        .add_systems(
            Update,
            (setup_ui, camera_controller::atlas_orbit_camera_input_map),
        )
        .run()
}

fn configure_visuals_system(mut contexts: EguiContexts) {
    let mut atlas_visuals = egui::Visuals::default();
    atlas_visuals.window_rounding = 0.0.into();
    atlas_visuals.window_fill = egui::Color32::from_black_alpha((255. * 0.9) as u8);
    atlas_visuals.window_stroke = egui::Stroke::new(0.2, egui::Color32::from_white_alpha(255));
    atlas_visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;
    atlas_visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    atlas_visuals.widgets.active.bg_fill = Color32::TRANSPARENT;
    atlas_visuals.widgets.hovered.bg_fill = Color32::TRANSPARENT;
    atlas_visuals.menu_rounding = 0.0.into();
    contexts.ctx_mut().set_visuals(atlas_visuals);
}

fn lerp_color(start: egui::Color32, end: egui::Color32, _t: f64) -> egui::Color32 {
    let t = _t as f32;
    let start_alpha = start.a() as f32 / 255.0;
    let end_alpha = end.a() as f32 / 255.0;
    let alpha = (1.0 - t) * start_alpha + t * end_alpha;
    let alpha = alpha.clamp(0.0, 1.0) * 255.0;

    let start_red = start.r() as f32 / 255.0;
    let end_red = end.r() as f32 / 255.0;
    let red = (1.0 - t) * start_red + t * end_red;
    let red = (red.clamp(0.0, 1.0) * 255.0) as u8;

    let start_green = start.g() as f32 / 255.0;
    let end_green = end.g() as f32 / 255.0;
    let green = (1.0 - t) * start_green + t * end_green;
    let green = (green.clamp(0.0, 1.0) * 255.0) as u8;

    let start_blue = start.b() as f32 / 255.0;
    let end_blue = end.b() as f32 / 255.0;
    let blue = (1.0 - t) * start_blue + t * end_blue;
    let blue = (blue.clamp(0.0, 1.0) * 255.0) as u8;

    egui::Color32::from_rgba_premultiplied(red, green, blue, alpha as u8)
}

fn setup_ui(
    time: Res<Time>,
    mut atlas_diagnostics: ResMut<AtlasDiagnostics>,
    mut napkin: ResMut<NapkinSettings>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
) {
    let ctx = contexts.ctx_mut();
    let primary_window = windows.single_mut();
    atlas_diagnostics.uptime += time.delta_seconds_f64();

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
    let atlas_window_frame = egui::Frame {
        fill: egui::Color32::from_black_alpha((255. * 0.9) as u8),
        inner_margin: egui::Margin {
            left: 4.,
            right: 4.,
            top: 4.,
            bottom: 4.,
        },
        stroke: egui::Stroke::new(0.2, Color32::from_white_alpha(255)),
        ..egui::Frame::none()
    };

    occupied_screen_space.bottom = egui::TopBottomPanel::bottom("footer_panel")
        .resizable(false)
        .frame(atlas_panel_frame)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label(format!("Projects: {}", napkin.projects.len()));
                    ui.label(format!("Nodes: {}", napkin.nodes.len()));
                    ui.label(format!("Edges: {}", napkin.edges.len()));
                    ui.label(format!("Node Metadata: {}", napkin.node_metadata.len()));
                    ui.label(format!("Edge Metadata: {}", napkin.edge_metadata.len()));
                });
                ui.add_space(20.0); // Spacing between sections
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Right Section");
                });
            });
        })
        .response
        .rect
        .height();

    occupied_screen_space.top = egui::TopBottomPanel::top("top_panel")
        .resizable(true)
        .frame(atlas_panel_frame)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Left side with menu buttons
                ui.menu_button("File", |ui| {
                    ui.menu_button("New", |ui| {
                        if ui.button("Node").clicked() {
                            // Logic for creating a new node
                        }
                        if ui.button("Edge").clicked() {
                            // Logic for creating a new edge
                        }
                    });
                });
                ui.menu_button("Edit", |_ui| {
                    // Placeholder for future edit options
                });

                ui.horizontal_centered(|ui| {
                    if ui.button("Perform Action").clicked() {
                        // Logic for the main action
                    }
                });

                // Right side with more placeholder text
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Placeholder").clicked() {}
                });
            });

            ui.collapsing("Project Details", |ui| {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            if let Some(project) = &napkin.projects.iter().find(|&project| {
                                Some(&project.id) == napkin.selected_project.as_ref()
                            }) {
                                format!("Current Project: @{}/{}", &project.scope, &project.name)
                            } else {
                                "Current Project: None".to_string()
                            },
                        )
                        .highlight();
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.small_button("X").clicked() {
                                napkin.selected_project = None;
                            }
                        });
                    });
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
                            }
                        })
                    });
                });
            });
        })
        .response
        .rect
        .height();

    occupied_screen_space.left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .frame(atlas_panel_frame)
        .show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.heading("Napkin Atlas");

                    ui.collapsing("Projects", |ui| {
                        let mut selected_project =
                            napkin.selected_project.clone().unwrap_or_default();
                        let mut selected_nodes = napkin.selected_nodes.clone().unwrap_or_default();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for project in napkin.projects.iter() {
                                if ui
                                    .button(format!("Project: @{}/{}", project.scope, project.name))
                                    .clicked()
                                {
                                    selected_project = project.id.clone();
                                    selected_nodes = Vec::new();
                                }
                            }
                        });
                        if Some(&selected_project) != napkin.selected_project.as_ref() {
                            napkin.selected_project = Some(selected_project.clone());
                        }
                    });

                    ui.collapsing("Nodes", |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::TOP),
                                    |ui| {
                                        if ui.button("+").clicked() {}
                                    },
                                )
                            });
                            for node in napkin.nodes.iter() {
                                if ui.button(format!("Node: {}", node.id)).clicked() {}
                            }
                        });
                    });

                    ui.collapsing("Edges", |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for node in napkin.nodes.iter() {
                                if ui.button(format!("Edge: {}", node.id)).clicked() {}
                            }
                        });
                    });
                },
            );
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();

    egui::Window::new("bottom_panel")
        .resizable(true)
        .frame(atlas_window_frame)
        .default_pos(egui::pos2(0.0, ctx.available_rect().bottom()))
        .show(ctx, |ui| {
            ui.group(|ui| {
                ui.label("Node/Edge/Metadata Viewer/Editor");
                ui.separator();
                if let Some(selected_nodes) = &napkin.selected_nodes {
                    for node in selected_nodes {
                        ui.horizontal(|ui| {
                            let node_title = napkin
                                .node_metadata
                                .iter()
                                .find(|metadata| metadata.owner_id == node.id)
                                .map_or(node.id.clone(), |metadata| {
                                    metadata.value["text"].as_str().unwrap_or("").to_string()
                                });
                            let project = napkin
                                .projects
                                .iter()
                                .find(|p| p.id == node.project)
                                .unwrap();
                            let project_display = format!("@{}/{}", project.scope, project.name);
                            ui.label(format!("Node ID: {}", node_title));
                            ui.label(format!("Project: {}", project_display));
                        });
                    }
                }
                if let Some(selected_edges) = &napkin.selected_edges {
                    for edge_id in selected_edges {
                        ui.horizontal(|ui| {
                            ui.label(format!("Edge ID: {}", edge_id));
                            // Placeholder for edge details
                        });
                    }
                }
                if let Some(node_metadata) = napkin.node_metadata.first() {
                    ui.horizontal(|ui| {
                        ui.label(format!("Metadata Name: {}", node_metadata.name));
                        ui.label(format!("Metadata Value: {}", node_metadata.value));
                    });
                }
            });
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });
    occupied_screen_space.right = egui::SidePanel::right("right_panel")
        .resizable(true)
        .frame(atlas_panel_frame)
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.vertical(|ui| {
                    let mut fps_data = Vec::new();
                    for (uptime_at, value) in atlas_diagnostics.fps_log.iter() {
                        fps_data.push([*uptime_at, { *value }]);
                    }
                    let line = Line::new(fps_data);
                    let line_color =
                        if let Some((_last_uptime, last_fps)) = atlas_diagnostics.fps_log.last() {
                            // Replace the lerp calls in your code with:
                            
                            if *last_fps > 60.0 * 0.9 {
                                lerp_color(
                                    egui::Color32::GREEN,
                                    egui::Color32::YELLOW,
                                    (60.0 - *last_fps) / (60.0 * 0.2),
                                )
                            } else if *last_fps > 60.0 * 0.7 {
                                lerp_color(
                                    egui::Color32::YELLOW,
                                    egui::Color32::RED,
                                    (60.0 * 0.9 - *last_fps) / (60.0 * 0.2),
                                )
                            } else {
                                egui::Color32::RED
                            }
                        } else {
                            egui::Color32::GRAY // Default color if no fps data is available
                        };
                    let line = line.color(line_color);
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(format!(
                                "{:.3}",
                                &atlas_diagnostics.uptime
                            )));
                            ui.label(egui::RichText::new("Uptime (sec):").strong());
                            if let Some(value) = diagnostics
                                .get(&FrameTimeDiagnosticsPlugin::FPS)
                                .and_then(|fps| fps.smoothed())
                            {
                                let uptime = atlas_diagnostics.uptime;
                                atlas_diagnostics.fps_log.push((uptime, value));
                                if atlas_diagnostics.fps_log.len() > 500 {
                                    atlas_diagnostics.fps_log.drain(0..1); // Clear out the oldest entries
                                }
                                ui.label(format!("{:.2}", value));
                            } else {
                                ui.label("N/A");
                            }
                            ui.label(egui::RichText::new("FPS:").strong());
                        });
                    });
                    Plot::new("my_plot")
                        .view_aspect(3.0)
                        .show_axes(Vec2b { x: false, y: false })
                        .label_formatter(|name, value| {
                            if !name.is_empty() {
                                format!("{}: {:.*}%", name, 1, value.y)
                            } else {
                                format!("Uptime {:.3}\n{:.2} FPS", value.x, value.y)
                            }
                        })
                        .show(ui, |plot_ui| plot_ui.line(line));

                    ui.horizontal(|ui| {
                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::TopDown),
                            |ui| {
                                ui.checkbox(&mut atlas_diagnostics.debug_mode, "Debug Mode");
                            },
                        );
                    });

                    ui.allocate_space(ui.available_size());
                });
            });
        })
        .response
        .rect
        .width();

    let toolbar_frame = egui::Frame {
        fill: egui::Color32::from_black_alpha((255. * 0.99) as u8),
        stroke: egui::Stroke::NONE,
        ..egui::Frame::none()
    };

    let window_width = primary_window.width();
    egui::Window::new("Add New")
        .frame(toolbar_frame)
        .id(egui::Id::new("toolbar"))
        .resizable(false)
        .movable(false)
        .title_bar(false)
        .fixed_pos(egui::pos2(
            window_width - occupied_screen_space.right,
            occupied_screen_space.top,
        ))
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.add(
                    egui::Button::new(egui::RichText::new("New Node").size(24.))
                        .frame(false)
                        .stroke(egui::Stroke::NONE)
                        .fill(Color32::TRANSPARENT),
                );
                ui.add(
                    egui::Button::new(egui::RichText::new("New Edge").size(24.))
                        .frame(false)
                        .stroke(egui::Stroke::NONE)
                        .fill(Color32::TRANSPARENT),
                );
            });
            // ui.menu_button(egui::RichText::new("+").heading(), |ui| {
            //     if ui.button("Node").clicked() {}
            //     if ui.button("Edge").clicked() {}
            //     if ui.button("Metadata").clicked() {}
            // });
        });
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
}
