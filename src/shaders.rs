use glam::Mat4;

use crate::textures::Texture;

/// Repsonsible for storing per-frame shader uniform values and copying them to
/// a GPU backed buffer accessible to shaders.
#[derive(Debug)]
pub struct PerFrameUniforms {
    buffer_data: PerFrameBufferData,
    gpu_buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl PerFrameUniforms {
    /// Create a new PerFrameUniforms object that initializes all WGPU resources.
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer_data = PerFrameBufferData {
            view_projection: Mat4::IDENTITY,
            time_elapsed_seconds: 0.0,
            _padding: [0.0; 3],
        };

        let gpu_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("per-frame buffer"),
                contents: bytemuck::bytes_of(&buffer_data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("per-frame bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("per-frame bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: gpu_buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer_data,
            gpu_buffer,
            bind_group_layout,
            bind_group,
        }
    }

    /// Set view projection matrix such that it will be sent to the GPU the next
    /// time `write_to_gpu()` is called.
    pub fn set_view_projection(&mut self, view_projection: glam::Mat4) {
        self.buffer_data.view_projection = view_projection;
    }

    /// Set time elapsed such that it will be sent to the GPU the next time
    /// `write_to_gpu()` is called.
    pub fn set_time_elapsed_seconds(&mut self, time_elapsed: std::time::Duration) {
        self.buffer_data.time_elapsed_seconds = time_elapsed.as_secs_f32();
    }

    /// Copy per frame uniform values from this structure to the GPU.
    pub fn write_to_gpu(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.gpu_buffer, 0, bytemuck::bytes_of(&[self.buffer_data]))
    }

    /// Get the WGPU bind group layout object which is required when creating a
    /// new render pipeline layout.
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Get the WGPU bind group which is required when activating a bind group
    /// during a render pass.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

/// The actual data that will be written to the per-frame GPU buffer. The data
/// is stored in a separate struct from `PerFrameUniforms` to control its
/// memory layout such that it can be trivially converted to a byte buffer for
/// copying to GPU memory.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PerFrameBufferData {
    pub view_projection: glam::Mat4,
    pub time_elapsed_seconds: f32,
    pub _padding: [f32; 3],
}

/// Responsible for storing per-model shader values used during a model rendering
/// pass.
pub struct PerModelUniforms {
    bind_group: wgpu::BindGroup,
    _texture: wgpu::Texture,
}

impl PerModelUniforms {
    pub fn new(
        device: &wgpu::Device,
        per_model_bind_group_layout: &wgpu::BindGroupLayout,
        texture: Texture,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("per-model bind group"), // TODO(scott): Append caller specified name
            layout: per_model_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    // 0: Diffuse texture 2d.
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    // 1: Diffuse texture sampler.
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        Self {
            bind_group,
            _texture: texture.texture,
        }
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

/// Mesh vertex.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
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
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress
                        + std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
