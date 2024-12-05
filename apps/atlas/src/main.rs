use std::time::Duration;

use bevy::sprite::Material2d;
use bevy::time::Stopwatch;
use bevy::{prelude::*, window::CursorGrabMode};
use bevy::window::PrimaryWindow;
use bevy_egui::egui::Color32;
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

use plugins::edge_controller::EdgeControllerPlugin;
use plugins::project_controller::ProjectControllerPlugin;
use plugins::{
    debug_controller::{DebugState, DebugControllerPlugin},
    camera_controller::{CameraController, CameraControllerPlugin},
    napkin_controller::NapkinPlugin,
    node_controller::NodeControllerPlugin,
};
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
    // napkin_crosshair: NapkinCrosshair,
    hovered_nodes: Option<Vec<NapkinNode>>,
    hovered_edges: Option<Vec<NapkinEdge>>,
    selected_project: Option<String>,
    selected_node: Option<String>,
    selected_edge: Option<String>,
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
            // napkin_crosshair: NapkinCrosshair::default(),
            hovered_nodes: None,
            hovered_edges: None,
            selected_node: None,
            selected_edge: None,
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
        .register_request_type::<Vec<NapkinEdgeMetadata>>()
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
            EdgeControllerPlugin,
            CameraControllerPlugin,
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

fn setup_ui(
    mut contexts: EguiContexts,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut napkin: ResMut<NapkinSettings>,
    mut edits: ResMut<NapkinEdits>,
    mut debug_state: ResMut<DebugState>,
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
            ui.separator();;
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
                        let project = napkin.projects.iter().find(|p| p.id == edits.edge.project).unwrap();
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

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();

    fn get_mouse_position(windows: Query<&Window, With<PrimaryWindow>>, cameras: Query<(&Camera, &GlobalTransform)>) -> Option<Vec2> {
        let window = windows.single();
        let Some(cursor_position) = window.cursor_position() else {
            return None;
        };
        let camera = cameras.single();
        let Some(mouse_position) = camera.0.viewport_to_world_2d(camera.1, cursor_position) else {
            return None;
        };
        Some(mouse_position)
    }

    let central_panel = egui::CentralPanel::default()
        .frame(egui::Frame::none().inner_margin(4.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    occupied_screen_space.bottom = ui.horizontal_wrapped(|ui| {
                        ui.label("Z90 Studios, LLC");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let mouse_query = get_mouse_position(windows, cameras);
                            if let Some(mouse_position) = mouse_query {
                                ui.label(format!("({:.2},{:.2})", mouse_position.x, mouse_position.y));
                            }
                        });
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
            ui.button("Node");
            ui.button("Edge");
        });
    });

    if central_panel.response.context_menu_opened() {
        napkin.context_menu_open = true;
    } else {
        napkin.context_menu_open = false;
    }
}
