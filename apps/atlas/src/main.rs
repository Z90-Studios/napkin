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

use plugins::{
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
            initialized: false,
            uptime: Stopwatch::default(),
            refresh_timer: Timer::new(Duration::from_secs(60), TimerMode::Repeating),
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
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.05)))
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
            NapkinPlugin,
            NodeControllerPlugin,
            CameraControllerPlugin,
        ))
        // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
        // or after the `EguiSet::BeginPass` system (which belongs to the `CoreSet::PreUpdate` set).
        .add_systems(Startup, (configure_visuals_system, setup_camera))
        .add_systems(Update, setup_ui)
        .run();
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        CameraController::default(),
    ));
}

pub fn configure_visuals_system(mut contexts: EguiContexts) {
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

fn setup_ui(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut napkin: ResMut<NapkinSettings>,
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

    occupied_screen_space.left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .frame(atlas_panel_frame)
        .show(ctx, |ui| {
            ui.label("Atlas 2d");
            ui.label(format!("Uptime: {}s", napkin.uptime.elapsed_secs()));
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
}
