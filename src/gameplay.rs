use std::time::Duration;

use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::camera::Camera;

/// Camera controller is that moves closer/further from the camera target, and
/// also optionally rotates in a way that's sort of like an orbital camera but
/// not really.
///
/// TODO(scott): Rewrite this as a real orbital camera.
pub struct CameraController {
    /// The number of units this controller moves per second in the direction
    /// it is facing.
    speed: f32,
    move_forward: bool,
    move_backward: bool,
    move_left: bool,
    move_right: bool,
}

impl CameraController {
    /// Create a new camera controller that moves `speed` units per second in the
    /// direction the camera is facing.
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
        }
    }

    /// Updates the camera controller state with the given input event. This
    /// method returns `true` if `event` was used by this update method, other
    /// -wise false is returned.
    pub fn process_input(&mut self, event: &WindowEvent) -> bool {
        match event {
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

    /// Applies updates to the camera that reflect the current state of this
    /// controller.
    pub fn update_camera(&self, camera: &mut Camera, _delta: Duration) {
        let forward = camera.target - camera.eye;
        let forward_dir = forward.normalize();
        let forward_distance = forward.length();

        // Move camera forward / backward. Take care not to glitch into the
        // center of the scene.
        if self.move_forward && forward_distance > self.speed {
            camera.eye += forward_dir * self.speed;
        }
        if self.move_backward {
            camera.eye -= forward_dir * self.speed;
        }

        // Left/right orbital motion. Recalculate the forward vector and
        // magnitude to account for forward/backward movement above. Direction
        // does not need to be re-calculated since forward movement does not
        // alter direction.
        let right = forward_dir.cross(camera.up);
        let forward = camera.target - camera.eye;
        let forward_distance = forward.length();

        // When rotating left or right, the distance between the target and eye
        // needs to be scaled. This keeps the eye positon on the circle made by
        // the target and eyhe.
        if self.move_right {
            camera.eye =
                camera.target - (forward + right * self.speed).normalize() * forward_distance;
        }

        if self.move_left {
            camera.eye =
                camera.target - (forward - right * self.speed).normalize() * forward_distance;
        }

        if self.move_right {}
    }
}

// TODO(scott): Simple tests for camera controller.
//  1. Move forward/backward/left/right: is new position, eye expected?
//  2. Does camera clamp the minimum/maximum forward?
