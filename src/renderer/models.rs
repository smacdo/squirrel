use std::{ops::Range, rc::Rc};

use glam::{Mat4, Quat, Vec3};

use super::{
    shaders::{BindGroupLayouts, PerModelUniforms, PerSubmeshUniforms},
    shading::Material,
    uniforms_buffers::UniformBuffer,
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
    uniforms: PerModelUniforms,
    /// Reference to the shared mesh that this model will draw.
    mesh: Rc<Mesh>,
}

impl Model {
    /// Create a new model.
    pub fn new(
        device: &wgpu::Device,
        layouts: &BindGroupLayouts,
        translation: Vec3,
        rotation: Quat,
        scale: Vec3,
        mesh: Rc<Mesh>,
    ) -> Self {
        let mut m = Self {
            translation: Default::default(),
            rotation: Default::default(),
            scale: Default::default(),
            uniforms: PerModelUniforms::new(device, layouts),
            mesh,
        };

        m.set_scale_rotation_translation(scale, rotation, translation);
        m
    }

    /// Get model uniforms.
    #[allow(dead_code)]
    pub fn uniforms(&mut self) -> &PerModelUniforms {
        &self.uniforms
    }

    /// Get model uniforms.
    pub fn uniforms_mut(&mut self) -> &mut PerModelUniforms {
        &mut self.uniforms
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

        self.uniforms
            .set_local_to_world(Mat4::from_scale_rotation_translation(
                scale,
                rotation,
                translation,
            ));
    }

    /// Set the position of this model.
    #[allow(dead_code)]
    pub fn set_translation(&mut self, translation: Vec3) {
        self.set_scale_rotation_translation(self.scale, self.rotation, translation)
    }

    /// Set the rotation of the model.
    #[allow(dead_code)]
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.set_scale_rotation_translation(self.scale, rotation, self.translation)
    }

    /// Set the scale of the model.
    #[allow(dead_code)]
    pub fn set_scale(&mut self, scale: Vec3) {
        self.set_scale_rotation_translation(scale, self.rotation, self.translation)
    }

    /// Prepare the model for rendering.
    pub fn prepare(&self, queue: &wgpu::Queue) {
        if self.uniforms.is_dirty() {
            self.uniforms.update_gpu(queue);
        }
    }
}

/// Mesh definition that is shared among one or more instances of model.
pub struct Mesh {
    /// A buffer storing this mesh's vertices.
    vertex_buffer: wgpu::Buffer,
    /// A buffer storing this mesh's indices.
    index_buffer: wgpu::Buffer,
    /// Submeshes that draw a portion of the total mesh.
    submeshes: Vec<Submesh>,
}

impl Mesh {
    pub fn new(
        vertex_buffer: wgpu::Buffer,
        index_buffer: wgpu::Buffer,
        index_count: u32,
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
            submeshes,
        }
    }
}

/// A subpart of a larger mesh which has its own shader uniforms.
pub struct Submesh {
    /// Uniform values associated with this submesh.
    uniforms: PerSubmeshUniforms,
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
        let uniforms = PerSubmeshUniforms::new(device, layouts, material);
        Self {
            uniforms,
            indices,
            base_vertex,
        }
    }
}

/// A trait for types that are capable of rendering models and meshes.
pub trait DrawModel<'a> {
    fn draw_model(&mut self, model: &'a Model);
    fn draw_mesh(&mut self, mesh: &'a Mesh);
}

impl<'rpass, 'a> DrawModel<'a> for wgpu::RenderPass<'rpass>
where
    'a: 'rpass,
{
    fn draw_model(&mut self, model: &'a Model) {
        // Bind the per-model uniforms for this model before drawing the mesh.
        debug_assert!(!model.uniforms.is_dirty());

        self.set_bind_group(1, model.uniforms.bind_group(), &[]);
        self.draw_mesh(&model.mesh);
    }

    fn draw_mesh(&mut self, mesh: &'a Mesh) {
        // Bind the mesh's vertex and index buffers.
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // Draw each sub-mesh in the mesh.
        for submesh in &mesh.submeshes {
            self.set_bind_group(2, submesh.uniforms.bind_group(), &[]);
            self.draw_indexed(submesh.indices.clone(), submesh.base_vertex, 0..1);
        }
    }
}
