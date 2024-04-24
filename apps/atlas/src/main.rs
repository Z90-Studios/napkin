use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui::{self, Color32}, EguiContexts, EguiPlugin};
use bevy_infinite_grid::{InfiniteGridPlugin, InfiniteGridBundle};

mod plugins;

use plugins::camera_controller::{self, CameraController, CameraControllerPlugin};

#[derive(Default, Resource)]
struct OccupiedScreenSpace {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

fn main() {
    App::new()
        .init_resource::<OccupiedScreenSpace>()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(CameraControllerPlugin)
        .add_plugins(InfiniteGridPlugin)
        .add_systems(Startup, (configure_visuals_system, setup_system))
        .add_systems(Update, (ui_example_system, camera_controller::run_camera_controller))
        .run()
}

fn configure_visuals_system(mut contexts: EguiContexts) {
    let atlas_visuals = egui::Visuals {
        window_rounding: 0.0.into(),
        window_fill: Color32::from_rgba_premultiplied(46, 64, 83, 40),
        window_stroke: contexts.ctx_mut().style().visuals.widgets.noninteractive.fg_stroke,
        ..Default::default()
    };
    contexts.ctx_mut().set_visuals(atlas_visuals);
}

fn ui_example_system(
    mut contexts: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
) {
    let ctx = contexts.ctx_mut();

    occupied_screen_space.left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
    occupied_screen_space.top = egui::TopBottomPanel::top("top_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();
    occupied_screen_space.bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();
    occupied_screen_space.right = egui::SidePanel::right("right_panel")
        .resizable(true)
        .show(ctx, |ui| {
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
}


fn setup_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Plane3d {
            normal: Direction3d::new(Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            }).unwrap()
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.3, 0.5, 0.3).into(),
            ..Default::default()
        }),
        ..Default::default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.7, 0.6).into(),
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_3)),
        ..Default::default()
    });

    // let camera_pos = Vec3::new(-2.0, 2.5, 5.0);
    // let camera_transform =
    //     Transform::from_translation(camera_pos).looking_at(CAMERA_TARGET, Vec3::Y);
    // commands.insert_resource(OriginalCameraTransform(camera_transform));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-1.0, 1.0, 1.0)
                .looking_at(Vec3::new(-1.0, 1.0, 0.0), Vec3::Y),
            ..default()
        },
        CameraController::default(),
    ));
    commands.spawn(InfiniteGridBundle {
        ..Default::default()
    });
}
