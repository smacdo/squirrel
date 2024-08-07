//! Rust representations of shader uniform buffers and other expected input
//! values. Each struct defined in this module has a matching struct of the same
//! name in shader code.
//!
//! Any struct representing a uniform buffer (eg `PerFrameBufferData`) must have
//! a memory layout that exactly matches the shader uniform buffer. In particular
//! all fields must be aligned to a 16 byte (eg `Vec4`) padding as this is a
//! WebGPU requirement.
mod packed_structs;

use glam::Vec4;
use packed_structs::{
    PackedDirectionalLight, PackedMaterialConstants, PackedPointLight, PackedSpotLight,
};

use super::{
    gpu_buffers::{DynamicGpuBuffer, GenericUniformBuffer, UniformBindGroup},
    lighting::{DirectionalLight, PointLight, SpotLight},
    materials::Material,
    textures,
};

// TODO(scott): Use a derive! macro to eliminate the copy-paste in these
//              `per-frame-*` structs.

/// The standard lighting shader used to render objects with Phong lighting.
///
/// NOTE: The following constants _must_ be kept in sync with the lit shader:
///  `MAX_POINT_LIGHTS`
pub mod lit_shader {
    /// The shader source code.
    pub const SHADER_CODE: &str = include_str!("shaders/lit_shader.wgsl");
    /// The maximum number of point lights that can be specified per model.
    pub const MAX_POINT_LIGHTS: usize = 4;
    pub const MAX_DIRECTIONAL_LIGHTS: usize = 3;
    pub const MAX_SPOT_LIGHTS: usize = 2;
}

/// Per-frame shader uniforms used by the standard shader model.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PerFramePackedUniforms {
    pub view_projection: glam::Mat4,
    pub view_pos: glam::Vec4,
    pub directional_lights: [PackedDirectionalLight; lit_shader::MAX_DIRECTIONAL_LIGHTS],
    pub spot_lights: [PackedSpotLight; lit_shader::MAX_SPOT_LIGHTS],
    pub directional_light_count: u32,
    pub spot_light_count: u32,
    pub output_is_srgb: u32,
    pub time_elapsed_seconds: f32,
}

pub struct PerFrameShaderVals {
    uniforms: GenericUniformBuffer<PerFramePackedUniforms>,
}

impl PerFrameShaderVals {
    /// Create a new per frame shader values struct. Only one instance is needed
    /// per renderer.
    pub fn new(device: &wgpu::Device, layouts: &BindGroupLayouts) -> Self {
        Self {
            uniforms: GenericUniformBuffer::<PerFramePackedUniforms>::new(
                device,
                Some("per-frame shader vals"),
                Default::default(),
                &layouts.per_frame_layout,
            ),
        }
    }

    /// Set view projection matrix.
    pub fn set_view_projection(&mut self, view_projection: glam::Mat4) {
        self.uniforms.values_mut().view_projection = view_projection;
    }

    /// Set the world space position of the camera.
    pub fn set_view_pos(&mut self, view_pos: glam::Vec3) {
        self.uniforms.values_mut().view_pos = Vec4::new(view_pos.x, view_pos.y, view_pos.z, 1.0);
    }

    /// Clear all lighting information.
    pub fn clear_lights(&mut self) {
        self.uniforms.values_mut().directional_light_count = 0;
        self.uniforms.values_mut().spot_light_count = 0;
    }

    /// Add directional light to the scene.
    pub fn add_directional_light(&mut self, light: &DirectionalLight) {
        let uniforms = self.uniforms.values_mut();

        debug_assert!(uniforms.directional_light_count < lit_shader::MAX_DIRECTIONAL_LIGHTS as u32);

        if uniforms.directional_light_count < lit_shader::MAX_DIRECTIONAL_LIGHTS as u32 {
            uniforms.directional_lights[uniforms.directional_light_count as usize] =
                light.clone().into();
            uniforms.directional_light_count += 1;
        }
    }

    /// Add a spot light to the scene.
    pub fn add_spot_light(&mut self, light: &SpotLight) {
        let uniforms = self.uniforms.values_mut();

        debug_assert!(uniforms.spot_light_count < lit_shader::MAX_SPOT_LIGHTS as u32);

        if uniforms.spot_light_count < lit_shader::MAX_SPOT_LIGHTS as u32 {
            uniforms.spot_lights[uniforms.spot_light_count as usize] = light.clone().into();
            uniforms.spot_light_count += 1;
        }
    }

    /// Set time elapsed in seconds.
    pub fn set_time_elapsed_seconds(&mut self, time_elapsed: std::time::Duration) {
        self.uniforms.values_mut().time_elapsed_seconds = time_elapsed.as_secs_f32();
    }

    /// Set if the output backbuffer format is SRGB or not.
    pub fn set_output_is_srgb(&mut self, is_srgb: bool) {
        self.uniforms.values_mut().output_is_srgb = if is_srgb { 1 } else { 0 };
    }

    /// Gets the bind group layout describing any instance of `PerFrameUniforms`.
    pub fn bind_group_layout_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
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
}

impl UniformBindGroup for PerFrameShaderVals {
    fn bind_group(&self) -> &wgpu::BindGroup {
        self.uniforms.bind_group()
    }
}

impl DynamicGpuBuffer for PerFrameShaderVals {
    fn update_gpu(&self, queue: &wgpu::Queue) {
        self.uniforms.update_gpu(queue)
    }

    fn is_dirty(&self) -> bool {
        self.uniforms.is_dirty()
    }
}

/// Per-model uniform values that are used by the standard shader model.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PerModelPackedUniforms {
    pub local_to_world: glam::Mat4,
    pub world_to_local: glam::Mat4,
    pub point_light: [PackedPointLight; lit_shader::MAX_POINT_LIGHTS],
    pub point_light_count: u32,
    pub _padding: [u32; 3],
}

/// Stores per-model shader values that are copied to the GPU prior to rendering
/// a model.
#[derive(Debug)]
pub struct PerModelShaderVals {
    uniforms: GenericUniformBuffer<PerModelPackedUniforms>,
}

impl PerModelShaderVals {
    /// Create a new PerModelShaderVals object. One instance per model.
    pub fn new(device: &wgpu::Device, layouts: &BindGroupLayouts) -> Self {
        Self {
            uniforms: GenericUniformBuffer::<PerModelPackedUniforms>::new(
                device,
                Some("per-model shader vals"),
                Default::default(),
                &layouts.per_model_layout,
            ),
        }
    }

    /// Set local to world transform matrix.
    #[allow(dead_code)]
    pub fn set_local_to_world(&mut self, local_to_world: glam::Mat4) {
        self.uniforms.values_mut().local_to_world = local_to_world;
        self.uniforms.values_mut().world_to_local = local_to_world.inverse();
        debug_assert!(!self.uniforms.values().world_to_local.is_nan());
    }

    /// Clear all lighting information.
    pub fn clear_lights(&mut self) {
        self.uniforms.values_mut().point_light_count = 0;
    }

    /// Add point light to the model.
    pub fn add_point_light(&mut self, light: &PointLight) {
        debug_assert!(light.ambient >= 0.0 && light.ambient <= 1.0);
        debug_assert!(light.specular >= 0.0 && light.specular <= 1.0);

        let uniforms = self.uniforms.values_mut();

        if uniforms.point_light_count < lit_shader::MAX_POINT_LIGHTS as u32 {
            debug_assert!(uniforms.point_light_count < lit_shader::MAX_POINT_LIGHTS as u32);

            uniforms.point_light[uniforms.point_light_count as usize] = light.clone().into();
            uniforms.point_light_count += 1;
        }
    }

    /// Gets the bind group layout describing any instance of `PerModelUniforms`.
    pub fn bind_group_layout_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
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
}

impl DynamicGpuBuffer for PerModelShaderVals {
    fn update_gpu(&self, queue: &wgpu::Queue) {
        self.uniforms.update_gpu(queue)
    }

    fn is_dirty(&self) -> bool {
        self.uniforms.is_dirty()
    }
}

impl UniformBindGroup for PerModelShaderVals {
    fn bind_group(&self) -> &wgpu::BindGroup {
        self.uniforms.bind_group()
    }
}

/// Per-submesh uniform values that are used by the standard shader model.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PerSubmeshPackedUniforms {
    pub material: PackedMaterialConstants,
}

/// Responsible for storing per-submesh shader values used during a submesh
/// rendering pass.
#[derive(Debug)]
pub struct PerSubmeshShaderVals {
    _tex_sampler: wgpu::Sampler,
    _diffuse_view: wgpu::TextureView,
    _specular_view: wgpu::TextureView,
    _emissive_view: wgpu::TextureView,
    uniforms: PerSubmeshPackedUniforms,
    gpu_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    is_dirty: std::cell::Cell<bool>,
}

impl PerSubmeshShaderVals {
    pub const UNIFORMS_BINDING_SLOT: u32 = 0;
    pub const SAMPLER_BINDING_SLOT: u32 = 1;
    pub const DIFFUSE_VIEW_BINDING_SLOT: u32 = 2;
    pub const SPECULAR_VIEW_BINDING_SLOT: u32 = 3;
    pub const EMISSIVE_VIEW_BINDING_SLOT: u32 = 4;

    pub fn new(device: &wgpu::Device, layouts: &BindGroupLayouts, material: &Material) -> Self {
        // TODO: How to move this into the GenericUniformBuffer type when we have
        // additional bind group entries for the textures?
        let tex_sampler = textures::create_default_sampler(device);
        let diffuse_view = material
            .diffuse_map
            .create_view(&wgpu::TextureViewDescriptor::default());
        let specular_view = material
            .specular_map
            .create_view(&wgpu::TextureViewDescriptor::default());
        let emissive_view = material
            .emissive_map
            .create_view(&wgpu::TextureViewDescriptor::default());

        let values = PerSubmeshPackedUniforms {
            material: material.clone().into(),
        };

        let gpu_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("per-submesh uniforms"),
                contents: bytemuck::bytes_of(&values),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("per-submesh bind group"), // TODO(scott): Append caller specified name
            layout: &layouts.per_submesh_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::UNIFORMS_BINDING_SLOT,
                    resource: gpu_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::SAMPLER_BINDING_SLOT,
                    resource: wgpu::BindingResource::Sampler(&tex_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: Self::DIFFUSE_VIEW_BINDING_SLOT,
                    resource: wgpu::BindingResource::TextureView(&diffuse_view),
                },
                wgpu::BindGroupEntry {
                    binding: Self::SPECULAR_VIEW_BINDING_SLOT,
                    resource: wgpu::BindingResource::TextureView(&specular_view),
                },
                wgpu::BindGroupEntry {
                    binding: Self::EMISSIVE_VIEW_BINDING_SLOT,
                    resource: wgpu::BindingResource::TextureView(&emissive_view),
                },
            ],
        });

        Self {
            _tex_sampler: tex_sampler,
            _diffuse_view: diffuse_view,
            _specular_view: specular_view,
            _emissive_view: emissive_view,
            uniforms: values,
            gpu_buffer,
            bind_group,
            is_dirty: std::cell::Cell::new(false),
        }
    }

    /// Gets the bind group layout describing any instance of `PerMeshUniforms`.
    ///
    /// Expected bind group inputs:
    ///  0 - uniforms
    ///  1 - texture map sampler
    ///  2 - diffuse texture
    ///  3 - specular texture
    ///  4 - emissive texture
    pub fn bind_group_layout_desc() -> wgpu::BindGroupLayoutDescriptor<'static> {
        wgpu::BindGroupLayoutDescriptor {
            label: Some("per-mesh bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: Self::UNIFORMS_BINDING_SLOT,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::SAMPLER_BINDING_SLOT,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::DIFFUSE_VIEW_BINDING_SLOT,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::SPECULAR_VIEW_BINDING_SLOT,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::EMISSIVE_VIEW_BINDING_SLOT,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        }
    }
}

impl DynamicGpuBuffer for PerSubmeshShaderVals {
    fn update_gpu(&self, queue: &wgpu::Queue) {
        self.is_dirty.swap(&std::cell::Cell::new(false));
        queue.write_buffer(&self.gpu_buffer, 0, bytemuck::bytes_of(&self.uniforms));
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty.get()
    }
}

impl UniformBindGroup for PerSubmeshShaderVals {
    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

/// A registry of bind group layouts used by this renderer.
#[derive(Debug)]
pub struct BindGroupLayouts {
    pub per_frame_layout: wgpu::BindGroupLayout,
    pub per_model_layout: wgpu::BindGroupLayout,
    pub per_submesh_layout: wgpu::BindGroupLayout,
}

impl BindGroupLayouts {
    /// Create a new bind group layout registry.
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            per_frame_layout: device
                .create_bind_group_layout(&PerFrameShaderVals::bind_group_layout_desc()),
            per_model_layout: device
                .create_bind_group_layout(&PerModelShaderVals::bind_group_layout_desc()),
            per_submesh_layout: device
                .create_bind_group_layout(&PerSubmeshShaderVals::bind_group_layout_desc()),
        }
    }
}

pub trait VertexLayout {
    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static>;
}
