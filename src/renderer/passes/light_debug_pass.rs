use glam::{Mat4, Quat, Vec3};
use wgpu::util::DeviceExt;

// TODO: Draw spot light as a pyramid mesh.
// TODO: Use model instancing for rendering the meshes.
// TODO: Re-use the existing cube mesh, just update the shader to ignore
//       unneeded attributes like normal.
// TODO: Add debug state to `DebugState`, then pass to here ::update + ::draw

use crate::renderer::{
    debug::{DebugVertex, CUBE_INDICES, CUBE_VERTS},
    gpu_buffers::{DynamicGpuBuffer, InstanceBuffer, UniformBindGroup},
    lighting::PointLight,
    scene::Scene,
    shaders::{BindGroupLayouts, PerFrameShaderVals},
};

/// Provides a debug visualization layer to the renderer.
///
/// Lighting information must be specified every frame as the information is not
/// retained between frames.
pub struct LightDebugPass {
    /// Render pipeline for the debug overlay.
    render_pipeline: wgpu::RenderPipeline,
    cube_vertex_buffer: wgpu::Buffer,
    cube_index_buffer: wgpu::Buffer,
    lamp_instances: DebugMeshInstanceBuffer,
    lamp_count: usize,
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
                    bind_group_layouts: &[&layouts.per_frame_layout],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    DebugVertex::desc(),
                    DebugMeshInstanceBuffer::vertex_layout(),
                ],
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
            lamp_instances: DebugMeshInstanceBuffer::new(device),
            lamp_count: 0,
        }
    }

    /// Set the world position of the scene light.
    pub fn add_point_light(&mut self, light: &PointLight) {
        self.lamp_instances
            .set_color_tint(self.lamp_count, light.color);
        self.lamp_instances.set_local_to_world(
            self.lamp_count,
            Mat4::from_scale_rotation_translation(
                Vec3::new(0.2, 0.2, 0.2),
                Quat::IDENTITY,
                light.position,
            ),
        );

        self.lamp_count += 1;
    }

    /// Prepare for rendering by creating and updating all resources used during
    /// rendering.
    pub fn prepare(&mut self, queue: &wgpu::Queue, scene: &Scene) {
        for light in &scene.point_lights {
            self.add_point_light(light);
        }

        if self.lamp_instances.is_dirty() {
            self.lamp_instances.update_gpu(queue)
        }
    }

    /// Draw the debug pass.
    pub fn draw(
        &self,
        output_view: &wgpu::TextureView,
        depth_buffer: &wgpu::TextureView,
        per_frame_uniforms: &PerFrameShaderVals, // TODO: Don't pass, move values to `prepare`.
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
        render_pass.set_vertex_buffer(1, self.lamp_instances.gpu_buffer_slice(..));
        render_pass.set_index_buffer(self.cube_index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..CUBE_INDICES.len() as u32, 0, 0..(self.lamp_count as u32));
    }

    pub fn finish_frame(&mut self) {
        self.lamp_count = 0
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DebugMeshPackedInstance {
    pub local_to_world: Mat4,
    pub color_tint: Vec3,
    pub _padding_1: f32,
}

#[derive(Debug)]
struct DebugMeshInstanceBuffer {
    buffer: InstanceBuffer<DebugMeshPackedInstance>,
}

impl DebugMeshInstanceBuffer {
    /// Create a new PerDebugMeshUniforms object. One instance per debug mesh.
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            buffer: InstanceBuffer::<DebugMeshPackedInstance>::new(
                device,
                Some("debug mesh instance buffer"),
                vec![
                    DebugMeshPackedInstance {
                        local_to_world: Default::default(),
                        color_tint: Vec3::ONE,
                        _padding_1: Default::default(),
                    };
                    100
                ],
            ),
        }
    }

    /// Set local to world transform matrix.
    pub fn set_local_to_world(&mut self, index: usize, local_to_world: glam::Mat4) {
        self.buffer.values_mut(index).local_to_world = local_to_world;
    }

    /// Set tint color.
    pub fn set_color_tint(&mut self, index: usize, color: glam::Vec3) {
        self.buffer.values_mut(index).color_tint = color;
    }

    /// Get the GPU buffer object used by this instance buffer.
    pub fn gpu_buffer_slice<S>(&self, bounds: S) -> wgpu::BufferSlice
    where
        S: std::ops::RangeBounds<wgpu::BufferAddress>,
    {
        self.buffer.gpu_buffer_slice(bounds)
    }

    pub fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<DebugMeshPackedInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // local_to_world: mat4 = 4 vec4
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // tint_color: vec4
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl DynamicGpuBuffer for DebugMeshInstanceBuffer {
    fn update_gpu(&self, queue: &wgpu::Queue) {
        self.buffer.update_gpu(queue)
    }

    fn is_dirty(&self) -> bool {
        self.buffer.is_dirty()
    }
}
