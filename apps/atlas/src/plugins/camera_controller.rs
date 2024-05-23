//! A freecam-style camera controller plugin.
//! To use in your own application:
//! - Copy the code for the [`CameraControllerPlugin`] and add the plugin to your App.
//! - Attach the [`CameraController`] component to an entity with a [`Camera3dBundle`].

use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_egui::egui::CursorIcon;
use bevy_egui::EguiContexts;
use smooth_bevy_cameras::controllers::orbit::{ControlEvent, OrbitCameraController};
use std::{f32::consts::*, fmt};

use crate::OccupiedScreenSpace;

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, run_camera_controller);
    }
}

/// Based on Valorant's default sensitivity, not entirely sure why it is exactly 1.0 / 180.0,
/// but I'm guessing it is a misunderstanding between degrees/radians and then sticking with
/// it because it felt nice.
pub const RADIANS_PER_DOT: f32 = 0.7 / 180.0;

#[derive(Component)]
pub struct CameraController {
    pub enabled: bool,
    pub initialized: bool,
    pub sensitivity: f32,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_run: KeyCode,
    pub mouse_key_cursor_grab: MouseButton,
    pub mouse_key_cursor_orbit: MouseButton,
    pub keyboard_key_toggle_cursor_grab: KeyCode,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub scroll_factor: f32,
    pub friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            initialized: false,
            sensitivity: 1.0,
            key_forward: KeyCode::KeyW,
            key_back: KeyCode::KeyS,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            key_up: KeyCode::KeyE,
            key_down: KeyCode::KeyQ,
            key_run: KeyCode::ShiftLeft,
            mouse_key_cursor_grab: MouseButton::Left,
            mouse_key_cursor_orbit: MouseButton::Right,
            keyboard_key_toggle_cursor_grab: KeyCode::Space,
            walk_speed: 5.0,
            run_speed: 15.0,
            scroll_factor: 0.1,
            friction: 0.5,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
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
    {:?}\t- Hold to grab cursor
    {:?}\t- Hold to orbit
    {:?}\t- Toggle cursor grab
    {:?} & {:?}\t- Fly forward & backwards
    {:?} & {:?}\t- Fly sideways left & right
    {:?} & {:?}\t- Fly up & down
    {:?}\t- Fly faster while held",
            self.mouse_key_cursor_grab,
            self.mouse_key_cursor_orbit,
            self.keyboard_key_toggle_cursor_grab,
            self.key_forward,
            self.key_back,
            self.key_left,
            self.key_right,
            self.key_up,
            self.key_down,
            self.key_run,
        )
    }
}

#[allow(clippy::too_many_arguments)]
pub fn run_camera_controller(
    time: Res<Time>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut mouse_events: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    occupied_screen_space: Res<OccupiedScreenSpace>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut toggle_cursor_grab: Local<bool>,
    mut mouse_cursor_grab: Local<bool>,
    mut mouse_cursor_orbit: Local<bool>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
) {
    let mut primary_window = windows.single_mut();
    let dt = time.delta_seconds();

    if let Ok((mut transform, mut controller)) = query.get_single_mut() {
        if !controller.initialized {
            let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
            controller.yaw = yaw;
            controller.pitch = pitch;
            controller.initialized = true;
            info!("{}", *controller);
        }
        if !controller.enabled {
            mouse_events.clear();
            return;
        }

        let mut scroll = 0.0;
        for scroll_event in scroll_events.read() {
            let amount = match scroll_event.unit {
                MouseScrollUnit::Line => scroll_event.y,
                MouseScrollUnit::Pixel => scroll_event.y / 16.0,
            };
            scroll += amount;
        }
        controller.walk_speed += scroll * controller.scroll_factor * controller.walk_speed;
        controller.run_speed = controller.walk_speed * 3.0;

        // Handle key input
        let mut axis_input = Vec3::ZERO;
        if key_input.pressed(controller.key_forward) {
            axis_input.z += 1.0;
        }
        if key_input.pressed(controller.key_back) {
            axis_input.z -= 1.0;
        }
        if key_input.pressed(controller.key_right) {
            axis_input.x += 1.0;
        }
        if key_input.pressed(controller.key_left) {
            axis_input.x -= 1.0;
        }
        if key_input.pressed(controller.key_up) {
            axis_input.y += 1.0;
        }
        if key_input.pressed(controller.key_down) {
            axis_input.y -= 1.0;
        }

        let mouse_border_offset = 5.0;

        if let (Some(mouse_position), window_height, window_width) = (
            primary_window.cursor_position(),
            primary_window.height(),
            primary_window.width(),
        ) {
            if (!*mouse_cursor_grab && !*toggle_cursor_grab)
                && (mouse_position.x < occupied_screen_space.left + mouse_border_offset
                    || mouse_position.x
                        > (window_width - occupied_screen_space.right - mouse_border_offset)
                    || mouse_position.y
                        > (window_height - occupied_screen_space.bottom - mouse_border_offset)
                    || mouse_position.y < occupied_screen_space.top + mouse_border_offset)
            {
                mouse_events.clear();
                return;
            }
        }

        let mut cursor_grab_change = false;
        if key_input.just_pressed(controller.keyboard_key_toggle_cursor_grab) {
            *toggle_cursor_grab = !*toggle_cursor_grab;
            cursor_grab_change = true;
        }
        if mouse_button_input.just_pressed(controller.mouse_key_cursor_grab) {
            *mouse_cursor_grab = true;
            cursor_grab_change = true;
        }
        if mouse_button_input.just_released(controller.mouse_key_cursor_grab) {
            *mouse_cursor_grab = false;
            cursor_grab_change = true;
        }
        if mouse_button_input.just_pressed(controller.mouse_key_cursor_orbit) {
            *mouse_cursor_orbit = true;
            cursor_grab_change = true;
        }
        if mouse_button_input.just_released(controller.mouse_key_cursor_orbit) {
            *mouse_cursor_orbit = false;
            cursor_grab_change = true;
        }
        let cursor_grab = *mouse_cursor_grab || *toggle_cursor_grab || *mouse_cursor_orbit;

        // Apply movement update
        if !*mouse_cursor_orbit && axis_input != Vec3::ZERO {
            let max_speed = if key_input.pressed(controller.key_run) {
                controller.run_speed
            } else {
                controller.walk_speed
            };
            controller.velocity = axis_input.normalize() * max_speed;
        } else {
            let friction = controller.friction.clamp(0.0, 1.0);
            controller.velocity *= 1.0 - friction;
            if controller.velocity.length_squared() < 1e-6 {
                controller.velocity = Vec3::ZERO;
            }
        }
        let forward = *transform.forward();
        let right = *transform.right();
        if !*mouse_cursor_orbit {
            transform.translation += controller.velocity.x * dt * right
                + controller.velocity.y * dt * Vec3::Y
                + controller.velocity.z * dt * forward;
        } else if *mouse_cursor_orbit {
            for mouse_event in mouse_events.read() {
                controller.yaw -=
                    mouse_event.delta.x * RADIANS_PER_DOT * controller.sensitivity;
                controller.pitch = (controller.pitch
                    - mouse_event.delta.y * RADIANS_PER_DOT * controller.sensitivity)
                    .clamp(-PI / 2., PI / 2.);
            }
            // Calculate the position relative to the origin
            let distance_from_origin = transform.translation.length();
            let orbit_position = Vec3::new(
                distance_from_origin * controller.yaw.cos() * controller.pitch.cos(),
                distance_from_origin * controller.pitch.sin(),
                distance_from_origin * controller.yaw.sin() * controller.pitch.cos(),
            );
            // Apply the orbit position to the camera's translation
            transform.translation = orbit_position;
        }

        // Handle cursor grab
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

        // Handle mouse input
        let mut mouse_delta = Vec2::ZERO;
        if cursor_grab {
            for mouse_event in mouse_events.read() {
                mouse_delta += mouse_event.delta;
            }
        } else {
            mouse_events.clear();
        }

        if mouse_delta != Vec2::ZERO {
            // Apply look update
            controller.pitch = (controller.pitch
                - mouse_delta.y * RADIANS_PER_DOT * controller.sensitivity)
                .clamp(-PI / 2., PI / 2.);
            controller.yaw -= mouse_delta.x * RADIANS_PER_DOT * controller.sensitivity;
            transform.rotation =
                Quat::from_euler(EulerRot::ZYX, 0.0, controller.yaw, controller.pitch);
        }
    }
}

pub fn atlas_orbit_camera_input_map(
    mut events: EventWriter<ControlEvent>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut contexts: EguiContexts,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    occupied_screen_space: Res<OccupiedScreenSpace>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    // keyboard: Res<ButtonInput<KeyCode>>,
    controllers: Query<&OrbitCameraController>,
) {
    let ctx = contexts.ctx_mut();
    let mut primary_window = primary_window.single_mut();
    // Can only control one camera at a time.
    let controller = if let Some(controller) = controllers.iter().find(|c| c.enabled) {
        controller
    } else {
        return;
    };
    let OrbitCameraController {
        mouse_rotate_sensitivity,
        
        mouse_wheel_zoom_sensitivity,
        pixels_per_line,
        ..
    } = *controller;

    let mouse_border_offset = 5.0;
    let cursor_inside = if let (Some(mouse_position), window_height, window_width) = (
        primary_window.cursor_position(),
        primary_window.height(),
        primary_window.width(),
    ) {
        let render_window_left = occupied_screen_space.left + mouse_border_offset;
        let render_window_right = window_width - occupied_screen_space.right - mouse_border_offset;
        let render_window_bottom = window_height - occupied_screen_space.bottom - mouse_border_offset;
        let render_window_top = occupied_screen_space.top + mouse_border_offset;

        mouse_position.x >= render_window_left
            && mouse_position.x <= render_window_right
            && mouse_position.y <= render_window_bottom
            && mouse_position.y >= render_window_top
    } else {
        false
    };

    if cursor_inside {
        if ctx.output(|o| o.cursor_icon) == CursorIcon::Default {
            // ctx.output_mut(|o| o.cursor_icon = CursorIcon::Grab);
        }
    } else {
        ctx.output_mut(|o| o.cursor_icon = CursorIcon::Default);
    }

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        cursor_delta += event.delta;
    }

    let mut cursor_grab = false;
    if mouse_buttons.pressed(MouseButton::Right) && cursor_inside {
        events.send(ControlEvent::Orbit(mouse_rotate_sensitivity * cursor_delta));
        cursor_grab = true;
    }

    let window_width = primary_window.width();
    let window_height = primary_window.height();

    if cursor_grab {
        primary_window.cursor.grab_mode = CursorGrabMode::Locked;
        primary_window.cursor.visible = false;
        primary_window
            .set_cursor_position(Some(Vec2::new(window_width / 2.0, window_height / 2.0)));
    } else {
        primary_window.cursor.grab_mode = CursorGrabMode::None;
        primary_window.cursor.visible = true;
    }

    let mut scalar = 1.0;
    for event in mouse_wheel_reader.read() {
        // scale the event magnitude per pixel or per line
        let scroll_amount = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / pixels_per_line,
        };
        scalar *= 1.0 - scroll_amount * mouse_wheel_zoom_sensitivity;
    }
    events.send(ControlEvent::Zoom(scalar));
}
