use glam::{Mat4, Quat, Vec3};
use wgpu::util::DeviceExt;

// TODO: Re-use the existing cube mesh, just update the shader to ignore
//       unneeded attributes like normal.
// TODO: Add debug state to `DebugState`, then pass to here ::update + ::draw

use crate::renderer::{
    debug::{DebugVertex, CUBE_INDICES, CUBE_VERTS},
    shaders::{BindGroupLayouts, PerDebugMeshUniforms, PerFrameUniforms},
    uniforms_buffers::UniformBuffer,
};

/// Provides a debug visualization layer to the renderer.
pub struct LightDebugPass {
    /// Render pipeline for the debug overlay.
    render_pipeline: wgpu::RenderPipeline,
    cube_vertex_buffer: wgpu::Buffer,
    cube_index_buffer: wgpu::Buffer,
    light_cube_uniforms: Vec<PerDebugMeshUniforms>, // TODO: Use model instancing.
}

impl LightDebugPass {
    const SHADER: &'static str = include_str!("debug_shader.wgsl");

    /// Create a new debug pass. Only one instance is needed per renderer.
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        layouts: &BindGroupLayouts,
    ) -> Self {
        // Load the cube debug mesh and generate N instances for rendering.
        let cube_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Debug Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(CUBE_VERTS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let cube_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Debug Cube Index Buffer"),
            contents: bytemuck::cast_slice(CUBE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let light_cube_uniforms = vec![PerDebugMeshUniforms::new(device, layouts)];

        // Load the shader used to render debug meshes.
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(Self::SHADER.into()),
        });

        // Create a render pipeline for rendering the debug layer.
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("debug pass render pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("debug pass pipeline layout"),
                    bind_group_layouts: &[
                        &layouts.per_frame_layout,
                        &layouts.per_debug_mesh_layout,
                    ],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[DebugVertex::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: super::DepthPass::DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less, // Fragments drawn front to back.
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Self {
            render_pipeline,
            cube_vertex_buffer,
            cube_index_buffer,
            light_cube_uniforms,
        }
    }

    /// Set the world position of the scene light.
    pub fn set_light_position(&mut self, position: Vec3) {
        self.light_cube_uniforms[0].set_local_to_world(Mat4::from_scale_rotation_translation(
            Vec3::new(0.2, 0.2, 0.2),
            Quat::IDENTITY,
            position,
        ))
    }

    /// Prepare for rendering by creating and updating all resources used during
    /// rendering.
    pub fn prepare(&mut self, queue: &wgpu::Queue) {
        for lcu in &self.light_cube_uniforms {
            if lcu.is_dirty() {
                lcu.update_gpu(queue);
            }
        }
    }

    /// Draw the debug pass.
    pub fn draw(
        &self,
        output_view: &wgpu::TextureView,
        depth_buffer: &wgpu::TextureView,
        per_frame_uniforms: &PerFrameUniforms, // TODO: Don't pass, move values to `prepare`.
        command_encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("debug render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_buffer,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, per_frame_uniforms.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.cube_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.cube_index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // Draw each debug cube mesh.
        for lcu in &self.light_cube_uniforms {
            render_pass.set_bind_group(1, lcu.bind_group(), &[]);
            render_pass.draw_indexed(0..CUBE_INDICES.len() as u32, 0, 0..1);
        }
    }
}
