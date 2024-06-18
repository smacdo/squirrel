use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Configurable state useful when debugging/testing the renderer.
#[derive(Default)]
pub struct DebugState {
    pub visualize_depth_pass: bool,
}

impl DebugState {
    pub fn process_input(&mut self, event: &winit::event::WindowEvent) {
        if let WindowEvent::KeyboardInput {
            event: keyboard_input_event,
            ..
        } = event
        {
            if keyboard_input_event.state == ElementState::Released {
                if let PhysicalKey::Code(KeyCode::KeyZ) = keyboard_input_event.physical_key {
                    self.visualize_depth_pass = !self.visualize_depth_pass;
                }
            }
        }
    }
}
