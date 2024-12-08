use std::time::Duration;

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::sprite::Material2d;
use bevy::time::Stopwatch;
use bevy::{prelude::*, window::CursorGrabMode};
use bevy::window::PrimaryWindow;
use bevy_egui::egui::{Color32, Vec2b};
use bevy_egui::{
    egui,
    egui::CursorIcon,
    EguiContexts,
    EguiPlugin,
};
use egui_plot::{Line, Plot};
use bevy_rapier2d::prelude::*;
use bevy_http_client::prelude::*;

mod types;
mod plugins;

use plugins::edge_controller::EdgeControllerPlugin;
use plugins::edge_metadata_controller::EdgeMetadataControllerPlugin;
use plugins::node_metadata_controller::NodeMetadataControllerPlugin;
use plugins::project_controller::ProjectControllerPlugin;
use plugins::{
    debug_controller::{DebugState, DebugControllerPlugin},
    camera_controller::{CameraController, CameraControllerPlugin},
    napkin_controller::NapkinPlugin,
    node_controller::NodeControllerPlugin,
};
use serde::{Deserialize, Serialize};
use types::napkin_types::*;

#[derive(Default, Resource)]
pub struct OccupiedScreenSpace {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

#[derive(Resource)]
pub struct NapkinSettings {
    server_url: String,
    initialized: bool,
    uptime: Stopwatch,
    refresh_timer: Timer,
    context_menu_open: bool,
    fps_log: Vec<(f64, f64)>,
    // napkin_crosshair: NapkinCrosshair,
    hovered_nodes: Option<Vec<NapkinNode>>,
    hovered_edges: Option<Vec<NapkinEdge>>,
    selected_project: Option<String>,
    selected_node: Option<String>,
    selected_node_metadata: Option<(String, String)>,
    selected_edge: Option<String>,
    selected_edge_metadata: Option<(String, String)>,
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
            initialized: false,
            uptime: Stopwatch::default(),
            refresh_timer: Timer::new(Duration::from_secs(60), TimerMode::Repeating),
            selected_project: None,
            context_menu_open: false,
            fps_log: Vec::new(),
            // napkin_crosshair: NapkinCrosshair::default(),
            hovered_nodes: None,
            hovered_edges: None,
            selected_node: None,
            selected_node_metadata: None,
            selected_edge: None,
            selected_edge_metadata: None,
            nodes: Vec::new(),
            node_metadata: Vec::new(),
            edges: Vec::new(),
            edge_metadata: Vec::new(),
            projects: Vec::new(),
            project_search_string: String::new(),
        }
    }
}

#[derive(Resource)]
pub struct NapkinEdits {
    project: NapkinProject,
    save_project: bool,
    node: NapkinNode,
    save_node: bool,
    edge: NapkinEdge,
    save_edge: bool,
    node_metadata: NapkinNodeMetadata,
    save_node_metadata: bool,
    edge_metadata: NapkinEdgeMetadata,
    save_edge_metadata: bool,
}

impl Default for NapkinEdits {
    fn default() -> Self {
        Self {
            project: NapkinProject::default(),
            save_project: false,
            node: NapkinNode::default(),
            save_node: false,
            edge: NapkinEdge::default(),
            save_edge: false,
            node_metadata: NapkinNodeMetadata::default(),
            save_node_metadata: false,
            edge_metadata: NapkinEdgeMetadata::default(),
            save_edge_metadata: false,
        }
    }
}

fn main() {
    App::new()
        .init_resource::<OccupiedScreenSpace>()
        .init_resource::<NapkinSettings>()
        .init_resource::<NapkinEdits>()
        .insert_resource(Msaa::Sample8)
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.05)))
        .register_request_type::<Vec<NapkinProject>>()
        .register_request_type::<NapkinProject>()
        .register_request_type::<Vec<NapkinNode>>()
        .register_request_type::<NapkinNode>()
        .register_request_type::<Vec<NapkinEdge>>()
        .register_request_type::<NapkinEdge>()
        .register_request_type::<Vec<NapkinNodeMetadata>>()
        .register_request_type::<NapkinNodeMetadata>()
        .register_request_type::<Vec<NapkinEdgeMetadata>>()
        .register_request_type::<NapkinEdgeMetadata>()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0))
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins((
            DebugControllerPlugin,
            HttpClientPlugin,
            NapkinPlugin,
            ProjectControllerPlugin,
            NodeControllerPlugin,
            NodeMetadataControllerPlugin,
            EdgeControllerPlugin,
            EdgeMetadataControllerPlugin,
            CameraControllerPlugin,
            FrameTimeDiagnosticsPlugin,
        ))
        // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
        // or after the `EguiSet::BeginPass` system (which belongs to the `CoreSet::PreUpdate` set).
        .add_systems(Startup, (configure_visuals_system, setup_camera))
        .add_systems(Update, setup_ui)
        .run();
}

pub fn setup_camera(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2dBundle::default(),
        CameraController::default(),
    ));
}

pub fn configure_visuals_system(mut contexts: EguiContexts) {
    let mut atlas_visuals = egui::Visuals::default();
    atlas_visuals.window_rounding = 0.0.into();
    atlas_visuals.window_fill = egui::Color32::from_black_alpha((255. * 0.9) as u8);
    atlas_visuals.window_stroke = egui::Stroke::new(0.4, egui::Color32::from_white_alpha(255));
    atlas_visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;
    atlas_visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    atlas_visuals.widgets.active.bg_fill = Color32::TRANSPARENT;
    atlas_visuals.widgets.hovered.bg_fill = Color32::TRANSPARENT;
    atlas_visuals.menu_rounding = 0.0.into();
    contexts.ctx_mut().set_visuals(atlas_visuals);
}

#[derive(Debug, Serialize, Deserialize)]
struct EditableMetadata {
    name: String,
    value: String,
}

impl From<NapkinNodeMetadata> for EditableMetadata {
    fn from(metadata: NapkinNodeMetadata) -> Self {
        Self {
            name: metadata.name,
            value: metadata.value.to_string(),
        }
    }
}

impl From<NapkinEdgeMetadata> for EditableMetadata {
    fn from(metadata: NapkinEdgeMetadata) -> Self {
        Self {
            name: metadata.name,
            value: metadata.value.to_string(),
        }
    }
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
    mut contexts: EguiContexts,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut napkin: ResMut<NapkinSettings>,
    mut edits: ResMut<NapkinEdits>,
    mut debug_state: ResMut<DebugState>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let ctx = contexts.ctx_mut();

    let atlas_panel_frame = egui::Frame {
        fill: egui::Color32::from_black_alpha((255. * 0.99) as u8),
        inner_margin: egui::Margin {
            left: 4.,
            right: 4.,
            top: 4.,
            bottom: 4.,
        },
        ..egui::Frame::none()
    };
    let atlas_window_frame = egui::Frame {
        fill: egui::Color32::from_black_alpha((255. * 0.99) as u8),
        inner_margin: egui::Margin {
            left: 4.,
            right: 4.,
            top: 4.,
            bottom: 4.,
        },
        stroke: egui::Stroke::new(0.2, Color32::from_white_alpha(255)),
        ..egui::Frame::none()
    };

    let metadata_editor = egui::Window::new("Metadata Editor")
        .frame(atlas_window_frame)
        .open(&mut (napkin.selected_node_metadata.is_some() || napkin.selected_edge_metadata.is_some()))
        .default_height(500.0)
        .show(ctx, |ui| {

            let mut metadata = if napkin.selected_node_metadata.is_some() { EditableMetadata::from(edits.node_metadata.clone()) } else { EditableMetadata::from(edits.edge_metadata.clone()) };
            ui.horizontal_wrapped(|ui| {
                ui.text_edit_singleline(&mut metadata.name);
                ui.code_editor(&mut metadata.value);
            });
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());

            if napkin.selected_node_metadata.is_some() {
                edits.node_metadata.name = metadata.name;
                let value_edit = serde_json::from_str::<serde_json::Value>(&metadata.value);
                if value_edit.is_ok() {
                    edits.node_metadata.value = value_edit.unwrap();
                }    
            } else {

            }
        });

    occupied_screen_space.top = egui::TopBottomPanel::top("top_panel")
        .frame(atlas_panel_frame)
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                ui.label("Project: Napkin");
                ui.separator();
                ui.label("Atlas");
                ui.separator();
                ui.menu_button("Debug", |ui| {
                    ui.checkbox(&mut debug_state.rapier_debug_enabled, "Debug Mode");
                    ui.label(format!("Uptime: {}s", napkin.uptime.elapsed_secs().floor()));
                });
            });
        })
        .response
        .rect
        .height();

    occupied_screen_space.right = egui::SidePanel::right("right_panel")
        .resizable(true)
        .min_width(250.0)
        .frame(atlas_panel_frame)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let selected_project = napkin
                    .projects
                    .iter()
                    .find(|p| napkin.selected_project == Some(p.id.clone()))
                    .as_ref()
                    .map_or(
                        "All Projects".to_string(),
                        |p| format!("@{}/{}", p.scope, p.name)
                    );
                ui.label(selected_project);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui| {
                    let clear_button = ui.small_button("Clear");
                    if clear_button.clicked() {
                        napkin.project_search_string = "".to_string();
                        napkin.selected_project = None;
                    }
                    ui.separator();
                });
            });
            ui.horizontal(|ui| {
                ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::singleline(&mut napkin.project_search_string)
                        .hint_text(egui::RichText::new("Search Projects..."))
                );
            });

            let filtered_projects = napkin
                .projects
                .iter()
                .filter(|project| {
                    format!("@{}/{}", project.scope, project.name)
                        .contains(&napkin.project_search_string)
                })
                .cloned()
                .collect::<Vec<NapkinProject>>();
            
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .auto_shrink(false)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        for project in filtered_projects {
                            let project_button = ui
                                .button(egui::RichText::new(format!(
                                    "@{}/{}",
                                    project.scope, project.name,
                                )));
                            if project_button.clicked() {
                                    napkin.selected_project = Some(project.id.clone());
                                    edits.project = project.clone();
                            }
                            if napkin.selected_project == Some(project.id) {
                                project_button.highlight();
                            }
                        }
                    })
                });
            ui.separator();
            egui::CollapsingHeader::new("Project Properties")
                .default_open(true)
                .show(ui, |ui| {
                    if napkin.selected_project.is_none() {
                        ui.label("No project selected");
                    } else {
                        if Some(edits.project.id.clone()) != napkin.selected_project {
                            for p in napkin.projects.iter() {
                                if napkin.selected_project == Some(p.id.clone()) {
                                    edits.project = p.clone();
                                }
                            }
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.add(egui::Label::new(egui::RichText::new(
                                        if edits.project.id.is_empty() { "New Project" } else { &edits.project.id }
                            ).small()));
                        });
                        egui::Grid::new("project_view")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Scope");
                                ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut edits.project.scope));
                                ui.end_row();

                                ui.label("Name");
                                ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut edits.project.name));
                                ui.end_row();
                            });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.button(if edits.project.id.is_empty() { "Create" } else { "Apply" })
                                .clicked() {
                                    edits.save_project = true;
                                }
                        });
                    }
                });
            ui.separator();
            egui::CollapsingHeader::new("Node Properties")
                .default_open(true)
                .show(ui, |ui| {
                    if napkin.selected_node.is_none() {
                        ui.label("No node selected");
                    } else {
                        if Some(edits.node.id.clone()) != napkin.selected_node {
                            for n in napkin.nodes.iter() {
                                if napkin.selected_node == Some(n.id.clone()) {
                                    edits.node = n.clone();
                                }
                            }
                        }
                        let project = napkin.projects.iter().find(|p| p.id == edits.node.project).unwrap();
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.add(egui::Label::new(egui::RichText::new(
                                        if edits.node.id.is_empty() { "New Node" } else { &edits.node.id }
                            ).small()));
                        });
                        egui::ComboBox::from_label(
                            if edits.node.project.is_empty() {
                                "Select Project"
                            } else {
                                "Change Project"
                            }
                        ).selected_text(
                            if edits.node.project.is_empty() {
                                format!("None")
                            } else {
                                format!("@{}/{}", project.scope, project.name)
                            }
                        ).show_ui(ui, |ui| {
                            for p in napkin.projects.iter() {
                                ui.selectable_value(&mut edits.node.project, p.id.clone(), format!("@{}/{}", p.scope, p.name));
                            }
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.button(if edits.node.id.is_empty() { "Create" } else { "Apply" })
                                .clicked() {
                                    edits.save_node = true;
                                }
                        });
                    }
                });
            ui.separator();
            egui::CollapsingHeader::new("Node Metadata")
                .default_open(true)
                .show(ui, |ui| {
                    if napkin.selected_node.is_none() {
                        ui.label("No node selected");
                    } else {
                        let node_metadata = napkin.
                            node_metadata
                            .iter()
                            .filter(|m|
                                m.owner_id == napkin.selected_node.clone().unwrap()
                            ).cloned().collect::<Vec<NapkinNodeMetadata>>();
                        for n_metadata in node_metadata {
                            if ui.button(format!("{}", n_metadata.name)).clicked() {
                                napkin.selected_edge_metadata = None;
                                napkin.selected_node_metadata = Some((n_metadata.owner_id.clone(), n_metadata.name.clone()));
                                edits.node_metadata = n_metadata;
                            }
                        }
                    }
                });
            ui.separator();
            egui::CollapsingHeader::new("Edge Properties")
                .default_open(true)
                .show(ui, |ui| {
                    if napkin.selected_edge.is_none() {
                        ui.label("No edge selected");
                    } else {
                        if Some(edits.edge.id.clone()) != napkin.selected_edge {
                            for e in napkin.edges.iter() {
                                if napkin.selected_edge == Some(e.id.clone()) {
                                    edits.edge = e.clone();
                                }
                            }
                        }
                        let default_project = NapkinProject {
                            id: "".to_string(),
                            scope: "Unknown".to_string(),
                            name: "Unknown".to_string()
                        };
                        let project = napkin.projects.iter().find(|p| p.id == edits.edge.project).unwrap_or(&default_project);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.add(egui::Label::new(egui::RichText::new(
                                        if edits.edge.id.is_empty() { "New Edge" } else { &edits.edge.id }
                            ).small()));
                        });
                        egui::ComboBox::from_label(
                            if edits.edge.project.is_empty() {
                                "Select Project"
                            } else {
                                "Change Project"
                            }
                        ).selected_text(
                            if edits.edge.project.is_empty() {
                                format!("None")
                            } else {
                                format!("@{}/{}", project.scope, project.name)
                            }
                        ).show_ui(ui, |ui| {
                            for p in napkin.projects.iter() {
                                ui.selectable_value(&mut edits.edge.project, p.id.clone(), format!("@{}/{}", p.scope, p.name));
                            }
                        });
                        egui::Grid::new("edge_view")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Source Node");
                                ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut edits.edge.source));
                                ui.end_row();

                                ui.label("Target Node");
                                ui.add_sized(ui.available_size(), egui::TextEdit::singleline(&mut edits.edge.target));
                                ui.end_row();
                            });


                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.button(if edits.edge.id.is_empty() { "Create" } else { "Apply" })
                                .clicked() {
                                    edits.save_edge = true;
                                }
                        });

                    }
                });
            ui.separator();
            egui::CollapsingHeader::new("Edge Metadata")
                .default_open(true)
                .show(ui, |ui| {
                    if napkin.selected_edge.is_none() {
                        ui.label("No edge selected");
                    } else {
                        let edge_metadata = napkin.
                            edge_metadata
                            .iter()
                            .filter(|m|
                                m.owner_id == napkin.selected_edge.clone().unwrap()
                            ).cloned().collect::<Vec<NapkinEdgeMetadata>>();
                        for n_metadata in edge_metadata {
                            if ui.button(format!("{}", n_metadata.name)).clicked() {
                                napkin.selected_node_metadata = None;
                                napkin.selected_edge_metadata = Some((n_metadata.owner_id.clone(), n_metadata.name.clone()));
                                edits.edge_metadata = n_metadata;
                            }
                        }
                    }
                });
            ui.separator();

            let mut fps_data = Vec::new();
            for (uptime_at, value) in napkin.fps_log.iter() {
                fps_data.push([*uptime_at, { *value }]);
            }
            let line = Line::new(fps_data);
            let line_color =
                if let Some((_last_uptime, last_fps)) = napkin.fps_log.last() {
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
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.vertical(|ui| {
                    if let Some(value) = diagnostics
                        .get(&FrameTimeDiagnosticsPlugin::FPS)
                        .and_then(|fps| fps.smoothed())
                    {
                        let uptime = napkin.uptime.elapsed_secs_f64();
                        napkin.fps_log.push((uptime, value));
                        if napkin.fps_log.len() > 500 {
                            napkin.fps_log.drain(0..1); // Clear out the oldest entries
                        }
                        ui.label(format!("{:.2}", value));
                    } else {
                        ui.label("N/A");
                    }
                    Plot::new("fps_log")
                        .view_aspect(3.0)
                        .show_axes(Vec2b { x: false, y: false })
                        .label_formatter(|name, value| {
                            if !name.is_empty() {
                                format!("{}: {:.*}", name, 1, value.y)
                            } else {
                                format!("Uptime {:.3}\n{:.2} FPS", value.x, value.y)
                            }
                        })
                        .show(ui, |plot_ui| plot_ui.line(line));
                });
                ui.add_space(ui.available_height());
            });
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();

    let central_panel = egui::CentralPanel::default()
        .frame(egui::Frame::none().inner_margin(4.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    occupied_screen_space.bottom = ui.horizontal_wrapped(|ui| {
                        ui.label("Z90 Studios, LLC");
                    }).response.rect.height();
                });
            });
        });

    central_panel.response.context_menu(|ui| {
        ui.set_max_width(150.0);

        ui.menu_button("Add", |ui| {
            if ui.button("Project").clicked() {
                edits.project = NapkinProject {
                    id: "".to_string(),
                    scope: "".to_string(),
                    name: "".to_string(),
                };
                napkin.selected_project = Some("".to_string());
            }
            if ui.button("Node").clicked() {
                edits.node = NapkinNode {
                    id: "".to_string(),
                    project: "".to_string(),
                };
                napkin.selected_node = Some("".to_string());
            }
            if ui.button("Edge").clicked() {
                edits.edge = NapkinEdge {
                    id: "".to_string(),
                    project: "".to_string(),
                    source: "".to_string(),
                    target: "".to_string(),
                };
                napkin.selected_edge = Some("".to_string());
            }
        });
    });

    if 
        metadata_editor.is_some_and(|m| m.response.hovered())
        || central_panel.response.context_menu_opened()
    {
        napkin.context_menu_open = true;
    } else {
        napkin.context_menu_open = false;
    }

}
