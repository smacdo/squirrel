use std::{cell::OnceCell, sync::OnceLock};

use glam::Mat4;

use super::textures::Texture;

// TODO: Refactor into a trait or some other reusable functionality because there
//       is a lot of overlap between PerFrameUniforms and PerModelUniforms?

/// Repsonsible for storing per-frame shader uniform values and copying them to
/// a GPU backed buffer accessible to shaders.
#[derive(Debug)]
pub struct PerFrameUniforms {
    buffer_data: PerFrameBufferData,
    gpu_buffer: wgpu::Buffer,
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("per-frame bind group"),
            layout: &Self::bind_group_layout(device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: gpu_buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer_data,
            gpu_buffer,
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

    /// Get this object's WGPU bind group.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    /// Gets the bind group layout that describing any instances of `PerFrameUniforms`.
    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        // TODO(scott): Use OnceLock<wgpu::BindGroupLayout> to create one instance only.
        //              For some reason this is having trouble on webasm need to investigate.
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        })
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

/// Repsonsible for storing per-model shader uniform values and copying them to
/// a GPU backed buffer accessible to shaders.
#[derive(Debug)]
pub struct PerModelUniforms {
    buffer_data: PerModelBufferData,
    gpu_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl PerModelUniforms {
    /// Create a new PerModelUniforms object that initializes all WGPU resources.
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer_data = PerModelBufferData {
            local_to_world: Mat4::IDENTITY,
        };

        let gpu_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("per-model buffer"), // TODO(scott): Append caller specified name
                contents: bytemuck::bytes_of(&buffer_data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("per-model bind group"),
            layout: &Self::bind_group_layout(device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: gpu_buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer_data,
            gpu_buffer,
            bind_group,
        }
    }

    /// Set local to world transform matrix such that it will be sent to the GPU
    /// the next time `write_to_gpu()` is called.
    pub fn set_view_projection(&mut self, local_to_world: glam::Mat4) {
        self.buffer_data.local_to_world = local_to_world;
    }

    /// Copy per frame uniform values from this structure to the GPU.
    pub fn write_to_gpu(&self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.gpu_buffer, 0, bytemuck::bytes_of(&[self.buffer_data]))
    }

    /// Get this object's WGPU bind group.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    /// Gets the bind group layout that describing any instances of `PerModelUniforms`.
    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        // TODO(scott): Use OnceLock<wgpu::BindGroupLayout> to create one instance only.
        //              For some reason this is having trouble on webasm need to investigate.
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("per-model bind group layout"),
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
        })
    }
}

/// The actual data that will be written to the per-model GPU buffer. The data
/// is stored in a separate struct from `PerModelUniforms` to control its
/// memory layout such that it can be trivially converted to a byte buffer for
/// copying to GPU memory.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PerModelBufferData {
    pub local_to_world: glam::Mat4,
    // TODO(scott): Lighting information.
}

/// Responsible for storing per-model shader values used during a model rendering
/// pass.
pub struct PerMeshUniforms {
    bind_group: wgpu::BindGroup,
    _texture: wgpu::Texture,
}

impl PerMeshUniforms {
    pub fn new(device: &wgpu::Device, texture: Texture) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("per-model bind group"), // TODO(scott): Append caller specified name
            layout: &Self::bind_group_layout(device),
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

    /// Get this object's WGPU bind group.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    /// Gets the bind group layout that describing any instances of `PerMeshUniforms`.
    ///
    /// Bind Group Inputs:
    ///  0 - diffuse texture
    ///  1 - diffuse sampler
    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        // TODO(scott): Use OnceLock<wgpu::BindGroupLayout> to create one instance only.
        //              For some reason this is having trouble on webasm need to investigate.
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("per-mesh bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    // 0: Diffuse texture 2d.
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // 1: Diffuse texture sampler.
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This needs to match the filterable field for the texture
                    // from above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
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
