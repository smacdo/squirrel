use wgpu::util::DeviceExt;

// TODO(scott): Render quads and text.
// TODO(scott): Add debug state to `DebugState`, then pass to here ::update + ::draw

use crate::renderer::{
    debug::{DebugVertex, CUBE_INDICES, CUBE_VERTS},
    shaders::{BindGroupLayouts, PerDebugMeshUniforms, PerFrameUniforms},
    uniforms_buffers::UniformBuffer,
};

/// Provides a debug visualization layer to the renderer.
pub struct DebugPass {
    /// Render pipeline for the debug overlay.
    render_pipeline: wgpu::RenderPipeline,
    cube_vertex_buffer: wgpu::Buffer,
    cube_index_buffer: wgpu::Buffer,
    _cube_mesh_uniforms: PerDebugMeshUniforms, // TODO: Use model instancing.
}

impl DebugPass {
    // TODO: Swap!
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

        let cube_mesh_uniforms = PerDebugMeshUniforms::new(device, layouts);

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
            depth_stencil: None,
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
            _cube_mesh_uniforms: cube_mesh_uniforms,
        }
    }

    /// Draw the debug pass.
    pub fn draw(
        &self,
        output_view: &wgpu::TextureView,
        per_frame_uniforms: &PerFrameUniforms, // TODO: Don't pass, recreate to remove dependency.
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
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, per_frame_uniforms.bind_group(), &[]);
        render_pass.set_vertex_buffer(0, self.cube_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.cube_index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        // Draw each debug cube mesh.
        //for 0..1 {
        //render_pass.set_bind_group(1, self.cube_mesh_uniforms.bind_group(), &[]);
        //render_pass.draw_indexed((0..CUBE_INDICES.len() as u32), 0, 0..1);
        //}
    }
}
