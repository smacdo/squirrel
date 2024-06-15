use std::{ops::Range, sync::Arc};

use glam::{Mat4, Quat, Vec3};

use super::{
    shaders::{BindGroupLayouts, PerModelUniforms, PerSubmeshUniforms},
    textures::Texture,
};

// TODO: Pass diffuse texture as a material.
// TODO: Support shared vertex/index buffers? Shared materials?

/// A model is an instance of a mesh with its own state. Models can be drawn by
/// the renderer.
pub struct Model {
    /// The world position of this model.
    position: Vec3,
    /// The rotation of this model.
    rotation: Quat,
    /// Shader uniform values associated with this model. The uniforms must be
    /// uploaded to the GPU after changes to position, rotation etc. This update
    /// must happen prior to drawing.
    uniforms: PerModelUniforms,
    /// A flag that is set to true after any state is changed and the new value
    /// has not been submitted to the GPU.
    pub dirty: bool,
    /// Reference to the shared mesh that this model will draw.
    mesh: Arc<Mesh>,
}

impl Model {
    /// Create a new model.
    pub fn new(
        device: &wgpu::Device,
        layouts: &BindGroupLayouts,
        position: Vec3,
        rotation: Quat,
        mesh: Arc<Mesh>,
    ) -> Self {
        Self {
            position,
            rotation,
            uniforms: PerModelUniforms::new(device, layouts),
            mesh,
            dirty: true,
        }
    }

    /// Set the rotation of the model.
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        self.dirty = true;
    }

    /// Send model state (position, rotation, etc) to the GPU.
    pub fn update_gpu(&mut self, queue: &wgpu::Queue) {
        let mut model = Mat4::from_translation(self.position);
        model *= Mat4::from_quat(self.rotation);

        self.uniforms.set_local_to_world(model);
        self.uniforms.write_to_gpu(queue);

        self.dirty = false
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
        diffuse_texture: Texture,
    ) -> Self {
        let uniforms = PerSubmeshUniforms::new(device, layouts, diffuse_texture);
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
        debug_assert!(
            !model.dirty,
            "model state changes have not been submitted to the GPU prior this draw request"
        );

        // Bind the per-model uniforms for this model before drawing the mesh.
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
