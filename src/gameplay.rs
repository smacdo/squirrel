use std::time::Duration;

use glam::{Quat, Vec2};
use winit::event::{ElementState, WindowEvent};

use crate::camera::Camera;

// NOTE: The camera can be janky when trying to scroll past min/max forward. It
//       is also prone to weird behavior when vertically panning near to parallel
//       with the up vector. I thought I worked the math out for the orbital
//       mechanics and limits, but clearly I need to sit down again to work out
//       why these behaviors are emerging near the limits despite being clamped.

// TODO(scott): Simple tests for camera controller.
//  1. Move forward/backward/left/right: is new position, eye expected?
//  2. Does camera clamp the minimum/maximum forward?

/// Experimental arc-ball camera controller. This controller uses the camera's
/// target as the pivot point, and allows both rotation and zooming. Zooming is
/// accomplished with the mouse wheel. Rotation is done by holding the mouse
/// button down and panning in the direction you wish to rotate.
pub struct CameraController {
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

impl CameraController {
    /// Create a new camera controller that lets users pan and zoom on a pivot
    /// point.
    pub fn new() -> Self {
        Self {
            horizontal_speed: 25.0,
            vertical_speed: 20.0,
            allow_mouse_look: false,
            mouse_motion: None,
            mouse_scroll: None,
            scroll_direction_modifier: -1.0,
            scroll_speed_modifier: 25.0,
            min_distance: 1.0,
            max_distance: Some(20.0),
        }
    }

    /// Updates the camera controller state with the given input event. This
    /// method returns `true` if `event` was used by this update method, other
    /// -wise false is returned.
    pub fn process_input(&mut self, event: &WindowEvent) -> bool {
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

    /// Accumulates mouse motion deltas until camera updates are applied in
    /// `update_camera`.
    pub fn process_mouse_motion(&mut self, delta: Vec2) {
        if self.allow_mouse_look {
            self.mouse_motion = Some(self.mouse_motion.unwrap_or_default() + delta);
        }
    }

    /// Accumulates mouse scroll wheel deltas until camera updates are applied in
    /// `update_camera`.
    pub fn process_mouse_wheel(&mut self, delta: Vec2) {
        self.mouse_scroll = Some(self.mouse_scroll.unwrap_or_default() + delta);
    }

    /// Applies updates to the camera that reflect the current state of this
    /// controller.
    pub fn update_camera(&mut self, camera: &mut Camera, delta: Duration) {
        let pivot = camera.target;
        let delta_secs = delta.as_secs_f32();

        // Convert the mouse motion to an amount of rotation. The height of the
        // viewport is 180 degrees, and the width of the viewport is 360 degrees.
        let x_view_angles = 2.0 * std::f32::consts::PI / camera.viewport_width;
        let y_view_angles = std::f32::consts::PI / camera.viewport_height;

        let x_angle = self.mouse_motion.unwrap_or_default().x
            * x_view_angles
            * self.horizontal_speed
            * delta_secs;
        let y_angle = self.mouse_motion.unwrap_or_default().y
            * y_view_angles
            * self.vertical_speed
            * delta_secs;

        // Rotate camera around the Y axis. (horizontal mouse movement).
        let x_rotation = Quat::from_axis_angle(camera.up, x_angle);
        let camera_pos_1 = x_rotation * (camera.eye - pivot) + pivot;

        // Regenerate the forward and right vectors after moving the camera.
        let forward = pivot - camera_pos_1;
        let right = forward.normalize().cross(camera.up);

        // Rotate camera around the X axis (vertical mouse movement).
        let y_rotation = Quat::from_axis_angle(right, y_angle);
        let camera_pos_2 = y_rotation * (camera_pos_1 - pivot) + pivot;

        // Do not use the vertical rotation contribution if it causes the
        // camera to become nearly parallel with the camera's -+ up vector.
        let forward = (pivot - camera_pos_1).normalize();
        let cos_angle = forward.dot(camera.up);

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
        camera.eye = camera_pos;
        camera.target = pivot;

        // Reset update state.
        self.mouse_motion = None;
        self.mouse_scroll = None;
    }
}
