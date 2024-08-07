use std::{cell::Cell, ops::Range, rc::Rc};

use glam::{Quat, Vec3};

use crate::renderer::gpu_buffers::UniformBindGroup;

use super::{
    materials::Material,
    shaders::{BindGroupLayouts, PerModelShaderVals, PerSubmeshShaderVals, VertexLayout},
    ModelShaderValsKey,
};

// TODO: Pass diffuse texture as a material.
// TODO: Support shared vertex/index buffers? Shared materials?

/// A model is an instance of a mesh with its own state. Models can be drawn by
/// the renderer.
#[allow(dead_code)]
pub struct Model {
    /// The world position of this model.
    translation: Vec3,
    /// The rotation of this model.
    rotation: Quat,
    /// The scale of this model.
    scale: Vec3,
    /// Shader uniform values associated with this model. The uniforms must be
    /// uploaded to the GPU after changes to position, rotation etc. This update
    /// must happen prior to drawing.
    pub model_sv_key: ModelShaderValsKey,
    /// Specifies if the translation, rotation or scale of the model has changed
    /// since the last time those values were copied to the shader_vals instance
    /// backing this model.
    model_sv_dirty: Cell<bool>,
    /// Reference to the shared mesh that this model will draw.
    mesh: Rc<Mesh>,
}

impl Model {
    /// Create a new model.
    pub fn new(
        model_shader_vals: ModelShaderValsKey,
        mesh: Rc<Mesh>,
        translation: Vec3,
        rotation: Quat,
        scale: Vec3,
    ) -> Self {
        let mut m = Self {
            translation: Default::default(),
            rotation: Default::default(),
            scale: Default::default(),
            model_sv_key: model_shader_vals,
            model_sv_dirty: Cell::new(true), // Force an initial update.
            mesh,
        };

        m.set_scale_rotation_translation(scale, rotation, translation);
        m
    }

    /// Model translation offset.
    pub fn translation(&self) -> Vec3 {
        self.translation
    }

    /// Model rotation.
    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    /// Model scale.
    pub fn scale(&self) -> Vec3 {
        self.scale
    }

    /// Returns true if the values stored in this model (eg translation,
    /// rotation or scale) are out of date with respect to the values stored in
    /// the model's shader values uniform object.
    pub fn is_model_sv_dirty(&self) -> bool {
        self.model_sv_dirty.get()
    }

    /// Set position, rotation and scale of this model.
    #[allow(dead_code)]
    pub fn set_scale_rotation_translation(
        &mut self,
        scale: Vec3,
        rotation: Quat,
        translation: Vec3,
    ) {
        self.scale = scale;
        self.rotation = rotation;
        self.translation = translation;
        self.model_sv_dirty.replace(true);
    }

    /// Set the position of this model.
    #[allow(dead_code)]
    pub fn set_translation(&mut self, translation: Vec3) {
        self.translation = translation;
        self.model_sv_dirty.replace(true);
    }

    /// Set the rotation of the model.
    #[allow(dead_code)]
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.model_sv_dirty.replace(true);
    }

    /// Set the scale of the model.
    #[allow(dead_code)]
    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
        self.model_sv_dirty.replace(true);
    }

    /// Unsets the `model_sv_dirty`.
    ///
    /// This should only be called by the renderer after it has succesfully
    /// update the model's shader values uniform object.
    pub fn mark_model_sv_updated(&self) {
        self.model_sv_dirty.replace(false);
    }
}

/// Mesh definition that is shared among one or more instances of model.
pub struct Mesh {
    /// A buffer storing this mesh's vertices.
    vertex_buffer: wgpu::Buffer,
    /// A buffer storing this mesh's indices.
    index_buffer: wgpu::Buffer,
    /// Size of the index buffer eleents.
    index_format: wgpu::IndexFormat,
    /// Submeshes that draw a portion of the total mesh.
    submeshes: Vec<Submesh>,
}

impl Mesh {
    pub fn new(
        vertex_buffer: wgpu::Buffer,
        index_buffer: wgpu::Buffer,
        index_count: u32,
        index_format: wgpu::IndexFormat,
        submeshes: Vec<Submesh>,
    ) -> Self {
        assert!(
            index_count
                >= submeshes
                    .iter()
                    .map(|m| m.indices.end)
                    .max()
                    .unwrap_or_default(),
            "at least one submesh has index offsets larger than the associated index buffer"
        );

        Self {
            vertex_buffer,
            index_buffer,
            index_format,
            submeshes,
        }
    }

    pub fn index_format(&self) -> wgpu::IndexFormat {
        self.index_format
    }
}

/// A subpart of a larger mesh which has its own shader uniforms.
pub struct Submesh {
    /// Uniform values associated with this submesh.
    submesh_shader_vals: PerSubmeshShaderVals,
    /// The indices used when rendering this submesh.
    indices: Range<u32>,
    /// Base vertex used when rendering this submesh.
    base_vertex: i32,
}

impl Submesh {
    pub fn new(
        device: &wgpu::Device,
        layouts: &BindGroupLayouts,
        indices: Range<u32>,
        base_vertex: i32,
        material: &Material,
    ) -> Self {
        let uniforms = PerSubmeshShaderVals::new(device, layouts, material);
        Self {
            submesh_shader_vals: uniforms,
            indices,
            base_vertex,
        }
    }
}

/// A trait for types that are capable of rendering models and meshes.
pub trait DrawModel<'a> {
    fn draw_model(&mut self, model: &'a Model, model_sv: &'a PerModelShaderVals);
    fn draw_mesh(&mut self, mesh: &'a Mesh);
}

impl<'rpass, 'a> DrawModel<'a> for wgpu::RenderPass<'rpass>
where
    'a: 'rpass,
{
    fn draw_model(&mut self, model: &'a Model, model_sv: &'a PerModelShaderVals) {
        // Bind the per-model uniforms for this model before drawing the mesh.
        debug_assert!(!model.is_model_sv_dirty());

        self.set_bind_group(1, model_sv.bind_group(), &[]);
        self.draw_mesh(&model.mesh);
    }

    fn draw_mesh(&mut self, mesh: &'a Mesh) {
        // Bind the mesh's vertex and index buffers.
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), mesh.index_format());

        // Draw each sub-mesh in the mesh.
        for submesh in &mesh.submeshes {
            self.set_bind_group(2, submesh.submesh_shader_vals.bind_group(), &[]);
            self.draw_indexed(submesh.indices.clone(), submesh.base_vertex, 0..1);
        }
    }
}

/// Vertex format used by model meshes.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl VertexLayout for Vertex {
    /// Get a description of the vertex layout for wgpu.
    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
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
