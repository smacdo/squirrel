use super::{
    textures::Texture,
    uniforms_buffers::{GenericUniformBuffer, UniformBuffer},
};

/// Per-frame uniform values used by the standard shader model.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PerFrameBufferData {
    pub view_projection: glam::Mat4,
    pub time_elapsed_seconds: f32,
    pub output_is_srgb: u32,
    pub _padding: [f32; 2],
}

/// Repsonsible for storing per-frame shader uniform values and copying them to
/// a GPU backed buffer accessible to shaders.
pub struct PerFrameUniforms {
    pub buffer: GenericUniformBuffer<PerFrameBufferData>,
}

impl PerFrameUniforms {
    /// Create a new per frame uniform buffer. Only one instance is needed per
    /// renderer.
    pub fn new(device: &wgpu::Device, layouts: &BindGroupLayouts) -> Self {
        Self {
            buffer: GenericUniformBuffer::<PerFrameBufferData>::new(
                device,
                Some("per-frame uniforms"),
                Default::default(),
                &layouts.per_frame_layout,
            ),
        }
    }

    /// Set view projection matrix such that it will be sent to the GPU the next
    /// time `write_to_gpu()` is called.
    pub fn set_view_projection(&mut self, view_projection: glam::Mat4) {
        self.buffer.values_mut().view_projection = view_projection;
    }

    /// Set time elapsed such that it will be sent to the GPU the next time
    /// `write_to_gpu()` is called.
    pub fn set_time_elapsed_seconds(&mut self, time_elapsed: std::time::Duration) {
        self.buffer.values_mut().time_elapsed_seconds = time_elapsed.as_secs_f32();
    }

    /// Set if the output backbuffer format is SRGB or not.
    pub fn set_output_is_srgb(&mut self, is_srgb: bool) {
        self.buffer.values_mut().output_is_srgb = if is_srgb { 1 } else { 0 };
    }
}

impl UniformBuffer for PerFrameUniforms {
    fn update_gpu(&self, queue: &wgpu::Queue) {
        self.buffer.update_gpu(queue)
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        self.buffer.bind_group()
    }

    fn is_dirty(&self) -> bool {
        self.buffer.is_dirty()
    }
}

/// Per-model uniform values that are used by the standard shader model.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PerModelBufferData {
    pub local_to_world: glam::Mat4,
    // TODO(scott): Lighting information.
}

/// Repsonsible for storing per-model shader uniform values and copying them to
/// a GPU backed buffer accessible to shaders.
#[derive(Debug)]
pub struct PerModelUniforms {
    pub buffer: GenericUniformBuffer<PerModelBufferData>,
}

impl PerModelUniforms {
    /// Create a new PerModelUniforms object. One instance per model.
    pub fn new(device: &wgpu::Device, layouts: &BindGroupLayouts) -> Self {
        Self {
            buffer: GenericUniformBuffer::<PerModelBufferData>::new(
                device,
                Some("per-model uniforms"),
                Default::default(),
                &layouts.per_model_layout,
            ),
        }
    }

    /// Set local to world transform matrix such that it will be sent to the GPU
    /// the next time `write_to_gpu()` is called.
    pub fn set_local_to_world(&mut self, local_to_world: glam::Mat4) {
        self.buffer.values_mut().local_to_world = local_to_world;
    }
}

impl UniformBuffer for PerModelUniforms {
    fn update_gpu(&self, queue: &wgpu::Queue) {
        self.buffer.update_gpu(queue)
    }

    fn bind_group(&self) -> &wgpu::BindGroup {
        self.buffer.bind_group()
    }

    fn is_dirty(&self) -> bool {
        self.buffer.is_dirty()
    }
}

/// Responsible for storing per-submesh shader values used during a submesh
/// rendering pass.
pub struct PerSubmeshUniforms {
    bind_group: wgpu::BindGroup,
}

impl PerSubmeshUniforms {
    pub fn new(device: &wgpu::Device, layouts: &BindGroupLayouts, texture: Texture) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("per-model bind group"), // TODO(scott): Append caller specified name
            layout: &layouts.per_submesh_layout,
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

        Self { bind_group }
    }

    /// Get this object's WGPU bind group.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

/// A registry of bind group layouts used by this renderer.
pub struct BindGroupLayouts {
    pub per_frame_layout: wgpu::BindGroupLayout,
    pub per_model_layout: wgpu::BindGroupLayout,
    pub per_submesh_layout: wgpu::BindGroupLayout,
}

impl BindGroupLayouts {
    /// Create a new bind group layout registry.
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            per_frame_layout: device.create_bind_group_layout(&Self::per_frame_desc()),
            per_model_layout: device.create_bind_group_layout(&Self::per_model_desc()),
            per_submesh_layout: device.create_bind_group_layout(&Self::per_submesh_desc()),
        }
    }

    /// Gets the bind group layout describing any instance of `PerFrameUniforms`.
    pub fn per_frame_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
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
        }
    }

    /// Gets the bind group layout describing any instance of `PerModelUniforms`.
    pub fn per_model_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
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
        }
    }

    /// Gets the bind group layout describing any instance of `PerMeshUniforms`.
    ///
    /// Expected bind group inputs:
    ///  0 - diffuse texture
    ///  1 - diffuse sampler
    pub fn per_submesh_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
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
        }
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
