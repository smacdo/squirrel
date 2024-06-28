use std::time::Duration;

use glam::{Quat, Vec2, Vec3};
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::camera::Camera;

// NOTE: The camera can be janky when trying to scroll past min/max forward. It
//       is also prone to weird behavior when vertically panning near to parallel
//       with the up vector. I thought I worked the math out for the orbital
//       mechanics and limits, but clearly I need to sit down again to work out
//       why these behaviors are emerging near the limits despite being clamped.

// TODO(scott): Simple tests for camera controller.
//  1. Move forward/backward/left/right: is new position, eye expected?
//  2. Does camera clamp the minimum/maximum forward?

pub trait CameraController {
    /// Updates the camera controller state with the given input event. This
    /// method returns `true` if `event` was used by this update method, other
    /// -wise false is returned.
    fn process_input(&mut self, event: &WindowEvent) -> bool;

    /// Accumulates mouse motion deltas until camera updates are applied in
    /// `update_camera`.
    fn process_mouse_motion(&mut self, delta: Vec2);

    /// Accumulates mouse scroll wheel deltas until camera updates are applied in
    /// `update_camera`.
    fn process_mouse_wheel(&mut self, delta: Vec2);

    /// Applies updates to the camera that reflect the current state of this
    /// controller.
    fn update_camera(&mut self, camera: &mut Camera, delta: Duration);
}

/// A first person camera that moves in the direction the mouse is looking.
pub struct FreeLookCameraController {
    /// A movement speed modifier.
    move_speed: f32,
    look_speed: f32,
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
    mouse_delta: Option<Vec2>,
    pitch_deg: f32,
    yaw_deg: f32,
    scroll_wheel_delta: Option<f32>,
    fov_y: f32,
}

impl FreeLookCameraController {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            move_speed: 4.0,
            look_speed: 4.0,
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            mouse_delta: None,
            pitch_deg: 0.0,
            yaw_deg: -90.0,
            scroll_wheel_delta: None,
            fov_y: 45.0,
        }
    }
}

impl CameraController for FreeLookCameraController {
    fn process_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            // Keyboard input.
            WindowEvent::KeyboardInput {
                event: keyboard_input_event,
                ..
            } => {
                // Is the button pushed down or no longer down?
                let is_pressed = keyboard_input_event.state == ElementState::Pressed;

                match keyboard_input_event.physical_key {
                    PhysicalKey::Code(KeyCode::ArrowUp) | PhysicalKey::Code(KeyCode::KeyW) => {
                        self.move_forward = is_pressed;
                        true
                    }
                    PhysicalKey::Code(KeyCode::ArrowDown) | PhysicalKey::Code(KeyCode::KeyS) => {
                        self.move_backward = is_pressed;
                        true
                    }
                    PhysicalKey::Code(KeyCode::ArrowLeft) | PhysicalKey::Code(KeyCode::KeyA) => {
                        self.move_left = is_pressed;
                        true
                    }
                    PhysicalKey::Code(KeyCode::ArrowRight) | PhysicalKey::Code(KeyCode::KeyD) => {
                        self.move_right = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn process_mouse_motion(&mut self, delta: Vec2) {
        self.mouse_delta = Some(self.mouse_delta.unwrap_or_default() + delta);
    }

    fn process_mouse_wheel(&mut self, delta: Vec2) {
        self.scroll_wheel_delta = Some(self.scroll_wheel_delta.unwrap_or_default() + delta.x);
    }

    fn update_camera(&mut self, camera: &mut Camera, delta: Duration) {
        let mut camera_pos = camera.eye();
        let delta_secs = delta.as_secs_f32();
        let move_speed = self.move_speed * delta_secs;

        // Respond to keyboard forward/backward/left/right movement.
        if self.move_forward {
            camera_pos += move_speed * camera.forward();
        }

        if self.move_backward {
            camera_pos -= move_speed * camera.forward();
        }

        if self.move_left {
            camera_pos -= move_speed * (Vec3::cross(camera.forward(), camera.up()));
        }

        if self.move_right {
            camera_pos += move_speed * (Vec3::cross(camera.forward(), camera.up()));
        }

        // Handle mouse look.
        let look_speed = self.look_speed * delta_secs;
        self.yaw_deg += look_speed * self.mouse_delta.unwrap_or_default().x;
        self.pitch_deg -= look_speed * self.mouse_delta.unwrap_or_default().y;

        if self.yaw_deg > 89.0 {
            self.yaw_deg = 89.0;
        } else if self.yaw_deg < -90.0 {
            self.yaw_deg = -89.0;
        }

        let yaw = self.yaw_deg.to_radians();
        let pitch = self.pitch_deg.to_radians();

        let look_dir = Vec3::new(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        )
        .normalize();

        camera.reorient(camera_pos, camera_pos + look_dir);

        // Handle zoom in/out by adjusting the field of view.
        // TODO: Add speed modifier and adjust by time delta.
        self.fov_y += self.scroll_wheel_delta.unwrap_or_default();

        self.fov_y = self.fov_y.clamp(1.0, 60.0);

        camera.set_fov_y(self.fov_y.to_radians());

        // Reset mouse state.
        self.mouse_delta = None;
        self.scroll_wheel_delta = None;
    }
}

/// Experimental arc-ball camera controller. This controller uses the camera's
/// target as the pivot point, and allows both rotation and zooming. Zooming is
/// accomplished with the mouse wheel. Rotation is done by holding the mouse
/// button down and panning in the direction you wish to rotate.
pub struct ArcballCameraController {
    /// Horizontal panning speed modifier.
    horizontal_speed: f32,
    /// Vertical panning speed modifier.
    vertical_speed: f32,
    /// Allows mouse motion to contribute to the camera controller when set to
    /// true, otherwise mouse motion is ignored.
    allow_mouse_look: bool,
    /// Amount of mouse motion this frame encoded as a delta from the last call
    /// to update.
    mouse_motion: Option<Vec2>,
    /// The amount of scroll units that the mouse has moved since the last call
    /// to update.
    mouse_scroll: Option<Vec2>,
    /// A direction modifier to apply to mouse scroll actions. This value should
    /// be 1.0 or -1.0.
    scroll_direction_modifier: f32,
    /// Adjusts the mouse wheel scroll speed by the given amount.
    scroll_speed_modifier: f32,
    /// Minimum view distance from target.
    min_distance: f32,
    /// Maximum view distance from target.
    max_distance: Option<f32>,
}

impl ArcballCameraController {
    /// Create a new camera controller that lets users pan and zoom on a pivot
    /// point.
    pub fn new() -> Self {
        Self {
            horizontal_speed: 25.0,
            vertical_speed: 25.0,
            allow_mouse_look: false,
            mouse_motion: None,
            mouse_scroll: None,
            scroll_direction_modifier: -1.0,
            scroll_speed_modifier: 25.0,
            min_distance: 1.0,
            max_distance: Some(20.0),
        }
    }
}

impl CameraController for ArcballCameraController {
    fn process_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            // Capture mouse input.
            WindowEvent::MouseInput {
                button: winit::event::MouseButton::Left,
                state,
                ..
            } => {
                self.allow_mouse_look = state == &ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    fn process_mouse_motion(&mut self, delta: Vec2) {
        if self.allow_mouse_look {
            self.mouse_motion = Some(self.mouse_motion.unwrap_or_default() + delta);
        }
    }

    fn process_mouse_wheel(&mut self, delta: Vec2) {
        self.mouse_scroll = Some(self.mouse_scroll.unwrap_or_default() + delta);
    }

    fn update_camera(&mut self, camera: &mut Camera, delta: Duration) {
        let pivot = camera.target();
        let delta_secs = delta.as_secs_f32();

        // Convert the mouse motion to an amount of rotation. The height of the
        // viewport is 180 degrees, and the width of the viewport is 360 degrees.
        let x_view_angles = 2.0 * std::f32::consts::PI / camera.viewport_width();
        let y_view_angles = std::f32::consts::PI / camera.viewport_height();

        let x_angle = self.mouse_motion.unwrap_or_default().x
            * x_view_angles
            * self.horizontal_speed
            * delta_secs;
        let y_angle = self.mouse_motion.unwrap_or_default().y
            * y_view_angles
            * self.vertical_speed
            * delta_secs;

        // Rotate camera around the Y axis. (horizontal mouse movement).
        let x_rotation = Quat::from_axis_angle(camera.up(), x_angle);
        let camera_pos_1 = x_rotation * (camera.eye() - pivot) + pivot;

        // Regenerate the forward and right vectors after moving the camera.
        let forward = pivot - camera_pos_1;
        let right = forward.normalize().cross(camera.up());

        // Rotate camera around the X axis (vertical mouse movement).
        let y_rotation = Quat::from_axis_angle(right, y_angle);
        let camera_pos_2 = y_rotation * (camera_pos_1 - pivot) + pivot;

        // Do not use the vertical rotation contribution if it causes the
        // camera to become nearly parallel with the camera's -+ up vector.
        let forward = (pivot - camera_pos_1).normalize();
        let cos_angle = forward.dot(camera.world_up());

        let camera_pos = if cos_angle * y_angle.signum() < 0.99 {
            // Both horizontal and vertical rotation.
            camera_pos_2
        } else {
            // Only horizontal rotation.
            camera_pos_1
        };

        // Move closer or further away from the target if requested by input.
        let scroll_amount =
            self.mouse_scroll.unwrap_or_default().x * self.scroll_direction_modifier;
        let camera_pos =
            camera_pos - forward * scroll_amount * self.scroll_speed_modifier * delta_secs;

        // Don't scroll too close or too far from the target.
        let pivot_to_camera = camera_pos - pivot;
        let distance = pivot_to_camera.length();

        let camera_pos = if distance <= self.min_distance {
            pivot_to_camera.normalize() * self.min_distance
        } else if self
            .max_distance
            .map_or_else(|| false, |max_distance| distance >= max_distance)
        {
            pivot_to_camera.normalize() * self.max_distance.unwrap()
        } else {
            camera_pos
        };

        // Update camera position and target.
        camera.reorient(camera_pos, pivot);

        // Reset update state.
        self.mouse_motion = None;
        self.mouse_scroll = None;
    }
}
