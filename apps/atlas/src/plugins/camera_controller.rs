use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_egui::egui::CursorIcon;
use bevy_egui::EguiContexts;
use std::{f32::consts::*, fmt};

use crate::OccupiedScreenSpace;

use super::edge_controller::HoveredEdge;
use super::node_controller::HoveredNode;

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, run_camera_controller);
    }
}

#[derive(Component)]
pub struct CameraController {
    pub enabled: bool,
    pub initialized: bool,
    pub sensitivity: f32,
    //pub key_up: KeyCode,
    //pub key_down: KeyCode,
    //pub key_left: KeyCode,
    //pub key_right: KeyCode,
    //pub key_run: KeyCode,
    pub mouse_key_cursor_grab: MouseButton,
    pub scroll_factor: f32,
    //pub walk_speed: f32,
    //pub run_speed: f32,
    pub friction: f32,
    pub velocity: Vec2,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            initialized: false,
            sensitivity: 1.0,
            //key_up: KeyCode::KeyW,
            //key_down: KeyCode::KeyS,
            //key_left: KeyCode::KeyA,
            //key_right: KeyCode::KeyD,
            //key_run: KeyCode::ShiftLeft,
            mouse_key_cursor_grab: MouseButton::Left,
            scroll_factor: 1.25,
            //walk_speed: 200.0,
            //run_speed: 500.0,
            friction: 0.5,
            velocity: Vec2::ZERO,
        }
    }
}

impl fmt::Display for CameraController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "
Freecam Controls:
    Mouse\t- Move camera orientation
    Scroll\t- Adjust movement speed
    {:?}\t- Hold to pan",
//    {:?} & {:?}\t- Move up & down
//    {:?} & {:?}\t- Move left & right
//    {:?}\t- Move faster while held",
            self.mouse_key_cursor_grab,
            //self.key_up,
            //self.key_down,
            //self.key_left,
            //self.key_right,
            //self.key_run,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_camera_controller(
    time: Res<Time>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut contexts: EguiContexts,
    mut mouse_events: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    occupied_screen_space: Res<OccupiedScreenSpace>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut mouse_cursor_grab: Local<bool>,
    mut query: Query<(&mut Transform, &mut CameraController, &mut OrthographicProjection), With<Camera>>,
    hovered_nodes: Query<&mut HoveredNode, Without<Camera>>,
    hovered_edges: Query<&mut HoveredEdge, Without<Camera>>,
) {
    let ctx = contexts.ctx_mut();
    let mut primary_window = windows.single_mut();
    let dt = time.delta_seconds();

    if let Ok((mut transform, mut controller, mut projection)) = query.get_single_mut() {
        if !controller.initialized {
            controller.initialized = true;
            info!("{}", *controller);
        }

        let cursor_grab = *mouse_cursor_grab;

        if !controller.enabled && !cursor_grab {
            mouse_events.clear();
            return;
        }

        
        //controller.run_speed = controller.walk_speed * 3.0;

        // Key input
        //let mut axis_input = Vec2::ZERO;
        //if key_input.pressed(controller.key_up) {
        //    axis_input.y += 1.0;
        //}
        //if key_input.pressed(controller.key_down) {
        //    axis_input.y -= 1.0;
        //}
        //if key_input.pressed(controller.key_right) {
        //    axis_input.x += 1.0;
        //}
        //if key_input.pressed(controller.key_left) {
        //    axis_input.x -= 1.0;
        //}

        let mouse_border_offset = 5.0;

        if let (Some(mouse_position), window_height, window_width) = (
            primary_window.cursor_position(),
            primary_window.height(),
            primary_window.width(),
        ) {
            if !*mouse_cursor_grab
                && (mouse_position.x < occupied_screen_space.left + mouse_border_offset
                    || mouse_position.x
                        > (window_width - occupied_screen_space.right - mouse_border_offset)
                    || mouse_position.y
                        > (window_height - occupied_screen_space.bottom - mouse_border_offset)
                    || mouse_position.y < (occupied_screen_space.top + mouse_border_offset))
            {
                mouse_events.clear();
                return;
            }
        }

        let mut scroll = 0.0;
        for scroll_event in scroll_events.read() {
            let amount = match scroll_event.unit {
                MouseScrollUnit::Line => scroll_event.y,
                MouseScrollUnit::Pixel => scroll_event.y / 16.0,
            };
            scroll += amount;
        }
        if scroll != 0.0 {
            if scroll > 0.0 {
                projection.scale /= controller.scroll_factor * 1.0;
            }
            if scroll < 0.0 {
                projection.scale *= controller.scroll_factor * 1.0;
            }
        }


        let mut cursor_grab_change = false;
        if mouse_button_input.just_pressed(controller.mouse_key_cursor_grab) {
            *mouse_cursor_grab = true;
            cursor_grab_change = true;
        }
        if mouse_button_input.just_released(controller.mouse_key_cursor_grab) {
            *mouse_cursor_grab = false;
            cursor_grab_change = true;
        }


        // Apply movement
        //if axis_input != Vec2::ZERO {
        //    let max_speed = if key_input.pressed(controller.key_run) {
        //        controller.run_speed
        //    } else {
        //        controller.walk_speed
        //    };
        //    controller.velocity = axis_input.normalize() * max_speed;
        //} else {
        //    let friction = controller.friction.clamp(0.0, 1.0);
        //    controller.velocity *= 1.0 - friction;
        //    if controller.velocity.length_squared() < 1e-6 {
        //        controller.velocity = Vec2::ZERO;
        //    }
        //}
        //let forward = *transform.up();
        //let right = *transform.right();
        //transform.translation.x += -controller.velocity.x * dt * right.x;
        //transform.translation.y += -controller.velocity.y * dt * forward.y;

        // Handle grab
        if cursor_grab_change {
            if cursor_grab {
                if !primary_window.focused {
                    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
                    primary_window.cursor.visible = false;
                }
            } else {
                primary_window.cursor.grab_mode = CursorGrabMode::None;
                primary_window.cursor.visible = true;
            }
        }

        // Handle mouse
        let mut mouse_delta = Vec2::ZERO;
        if cursor_grab {
            for mouse_event in mouse_events.read() {
                mouse_delta += mouse_event.delta * projection.scale.abs();
            }
            ctx.output_mut(|o| o.cursor_icon = CursorIcon::Move);
        } else {
            mouse_events.clear();
            //ctx.output_mut(|o| o.cursor_icon = CursorIcon::Default);
        }

        if mouse_delta != Vec2::ZERO {
            transform.translation.y += mouse_delta.y * controller.sensitivity;
            transform.translation.x -= mouse_delta.x * controller.sensitivity;
        }
    }
}
