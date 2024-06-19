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

/// A lightweight vertex used for drawing cubes, quads and other primitive
/// shapes to the screen.
///
/// A debug vertex only has position and a single set of texture coordinates.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DebugVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl DebugVertex {
    /// Vertex buffer format for `DebugVertex`.
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DebugVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
/// Vertices for a full screen quad in CCW order.
pub const QUAD_VERTS: &[DebugVertex] = &[
    DebugVertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    DebugVertex {
        position: [-1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    DebugVertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    DebugVertex {
        position: [-1.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
];

/// Indices for a full screen quad in CCW order.
pub const QUAD_INDICES: &[u16] = &[0, 1, 2, 2, 1, 3];
