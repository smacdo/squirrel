use glam::{Vec3, Vec3Swizzles, Vec4};

use super::{
    textures::Texture,
    uniforms_buffers::{GenericUniformBuffer, UniformBuffer},
};

// TODO(scott): Use a derive! macro to eliminate the copy-paste in these
//              `per-frame-*` structs.

/// Per-frame uniform values used by the standard shader model.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PerFrameBufferData {
    pub view_projection: glam::Mat4,
    pub view_pos: glam::Vec4,
    pub time_elapsed_seconds: f32,
    pub output_is_srgb: u32,
    pub _padding_2: [f32; 2],
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

    /// Set view projection matrix.
    pub fn set_view_projection(&mut self, view_projection: glam::Mat4) {
        self.buffer.values_mut().view_projection = view_projection;
    }

    /// Set the world space position of the camera.
    pub fn set_view_pos(&mut self, view_pos: glam::Vec3) {
        self.buffer.values_mut().view_pos = Vec4::new(view_pos.x, view_pos.y, view_pos.z, 1.0);
    }

    /// Set time elapsed in seconds.
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
    pub object_color: glam::Vec3,
    pub _padding_0: f32,
    pub light_color: glam::Vec3,
    pub _padding_1: f32,
    pub light_position: glam::Vec3,
    pub _padding_2: f32,
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

    /// Set local to world transform matrix.
    #[allow(dead_code)]
    pub fn set_local_to_world(&mut self, local_to_world: glam::Mat4) {
        self.buffer.values_mut().local_to_world = local_to_world;
    }

    /// Set the object color.
    pub fn set_object_color(&mut self, color: Vec3) {
        self.buffer.values_mut().object_color = color;
    }

    /// Set the light color.
    pub fn set_light_color(&mut self, color: Vec3) {
        self.buffer.values_mut().light_color = color;
    }

    /// Set the light world position.
    #[allow(dead_code)]
    pub fn set_light_position(&mut self, position: Vec3) {
        self.buffer.values_mut().light_position = position;
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
            label: Some("per-submesh bind group"), // TODO(scott): Append caller specified name
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

/// Per-model uniform values that are used by the debug shader model.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PerDebugMeshBufferData {
    pub local_to_world: glam::Mat4,
    pub color_tint: glam::Vec3,
    pub _padding: [f32; 1],
}

/// Repsonsible for storing per-debug-mesh shader uniform values and copying
/// them to a GPU backed buffer accessible to shaders.
#[derive(Debug)]
pub struct PerDebugMeshUniforms {
    pub buffer: GenericUniformBuffer<PerDebugMeshBufferData>,
}

impl PerDebugMeshUniforms {
    /// Create a new PerDebugMeshUniforms object. One instance per debug mesh.
    pub fn new(device: &wgpu::Device, layouts: &BindGroupLayouts) -> Self {
        Self {
            buffer: GenericUniformBuffer::<PerDebugMeshBufferData>::new(
                device,
                Some("per-debug-mesh uniforms"),
                PerDebugMeshBufferData {
                    local_to_world: Default::default(),
                    color_tint: Vec3::ONE,
                    _padding: Default::default(),
                },
                &layouts.per_debug_mesh_layout,
            ),
        }
    }

    /// Set local to world transform matrix.
    pub fn set_local_to_world(&mut self, local_to_world: glam::Mat4) {
        self.buffer.values_mut().local_to_world = local_to_world;
    }

    /// Set tint color.
    #[allow(dead_code)]
    pub fn set_color_tint(&mut self, color: glam::Vec3) {
        self.buffer.values_mut().color_tint = color;
    }
}

impl UniformBuffer for PerDebugMeshUniforms {
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

/// A registry of bind group layouts used by this renderer.
pub struct BindGroupLayouts {
    pub per_frame_layout: wgpu::BindGroupLayout,
    pub per_model_layout: wgpu::BindGroupLayout,
    pub per_submesh_layout: wgpu::BindGroupLayout,
    pub per_debug_mesh_layout: wgpu::BindGroupLayout,
}

impl BindGroupLayouts {
    /// Create a new bind group layout registry.
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            per_frame_layout: device.create_bind_group_layout(&Self::per_frame_desc()),
            per_model_layout: device.create_bind_group_layout(&Self::per_model_desc()),
            per_submesh_layout: device.create_bind_group_layout(&Self::per_submesh_desc()),
            per_debug_mesh_layout: device.create_bind_group_layout(&Self::per_debug_mesh_desc()),
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

    /// Gets the bind group layout describing any instance of `PerDebugMeshUniforms`.
    pub fn per_debug_mesh_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("per-debug-mesh bind group layout"),
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
}

/// Mesh vertex.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
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
