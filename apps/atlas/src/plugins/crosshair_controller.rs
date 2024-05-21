use bevy::{prelude::*, render::mesh::shape::Icosphere};
use smooth_bevy_cameras::{controllers::orbit::OrbitCameraController, LookTransform};
use std::fmt;

use crate::{NapkinCrosshair, NapkinCrosshairSelectionTypes, NapkinSettings};

use super::node_controller::NodeController;

pub struct CrosshairControllerPlugin;

impl Plugin for CrosshairControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_crosshair);
        app.add_systems(Update, run_crosshair_controller);
    }
}

#[derive(Component)]
pub struct CrosshairController {}

impl Default for CrosshairController {
    fn default() -> Self {
        Self {}
    }
}

impl fmt::Display for CrosshairController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Crosshair created")
    }
}

pub fn spawn_crosshair(
    mut commands: Commands,
    ass: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut napkin: ResMut<NapkinSettings>,
) {
    let crosshair_gltf = ass.load("crosshair.glb#Scene0");

    let material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.1, 0.1),
        ..Default::default()
    });

    commands.spawn((
        SceneBundle {
            scene: crosshair_gltf,
            transform: Transform::from_translation(Vec3::ZERO + Vec3::new(0.0, 0.1, 0.0))
                .with_scale(Vec3::splat(0.1)), // Scale down by 90%
            ..Default::default()
        },
        CrosshairController {},
    ));
}

pub fn run_crosshair_controller(
    mut napkin: ResMut<NapkinSettings>,
    time: Res<Time>,
    mut crosshair_query: Query<(&mut Transform, &CrosshairController), Without<Camera>>,
    nodes: Query<(&GlobalTransform, &NodeController)>,
    mut camera: Query<(&mut Transform, &mut LookTransform), With<Camera>>,
) {
    let rotation_speed = 1.0; // Rotation speed in radians per second
    let movement_speed = 5.0; // Movement speed units per second
    let mut set_default = false;

    if let Some(selected_id) = &napkin.napkin_crosshair.selected_id {
        if let Some(node) = nodes
            .iter()
            .find(|(_, node_controller)| node_controller.id == *selected_id)
        {
            let node_position = node.0.translation() + Vec3::new(0.0, 0.1, 0.0);
            for (mut transform, _) in crosshair_query.iter_mut() {
                let direction = node_position - transform.translation;
                let distance = direction.length();
                if distance > 0.001 {
                    // Threshold to stop movement
                    let dynamic_speed = movement_speed * distance; // Increase speed based on distance
                    transform.translation +=
                        direction.normalize() * dynamic_speed * time.delta_seconds().min(distance);
                    for (mut camera_transform, mut look_transform) in camera.iter_mut() {
                        camera_transform.translation += direction.normalize()
                            * dynamic_speed
                            * time.delta_seconds().min(distance);
                        look_transform.eye += direction.normalize()
                            * dynamic_speed
                            * time.delta_seconds().min(distance);
                        look_transform.target += direction.normalize()
                            * dynamic_speed
                            * time.delta_seconds().min(distance);
                    }
                }
            }
        } else {
            set_default = true;
        }
    } else {
        set_default = true;
    }

    if (set_default) {
        napkin.napkin_crosshair.selected_id = napkin
            .nodes
            .iter()
            .filter(|node| {
                napkin
                    .selected_project
                    .as_ref()
                    .map_or(true, |selected_project| &node.project == selected_project)
            })
            .next()
            .map(|node| node.id.clone());
    }

    for (mut transform, _) in crosshair_query.iter_mut() {
        let rotation_angle = rotation_speed * time.delta_seconds();
        transform.rotate(Quat::from_rotation_y(rotation_angle));
    }
}
