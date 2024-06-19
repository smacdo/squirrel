use wgpu::util::DeviceExt;

use crate::renderer::debug::{DebugVertex, QUAD_INDICES, QUAD_VERTS};

// TODO: Pass projection zNear/zFar values to depth shader.
// TODO: Pass quad location (eg full screen, or NE,NW,SW,SE corner)

/// Provides both the texture for the depth pass as well as an optional
/// render pipeline for visualizing the pass as a full screen quad.
pub struct DepthPass {
    /// The depth buffer written to by the GPU.
    depth_texture: wgpu::Texture,
    /// A view into the depth texture. Required by the renderer for writing into
    /// the depth buffer, and by the debug visualizer for displaying the depth
    /// buffer.
    depth_texture_view: wgpu::TextureView,
    /// Sampler required for reading from the depth buffer for visualization.
    depth_sampler: wgpu::Sampler,
    /// Bind group layout required by depth buffer visualization shader.
    bind_group_layout: wgpu::BindGroupLayout,
    /// Bind group (texture view, sampler and uniforms) required by depth buffer
    /// visualization shader.
    bind_group: wgpu::BindGroup,
    /// Vertices required for drawing a quad to the screen for visualization.
    vertex_buffer: wgpu::Buffer,
    /// Indices required for drawing a quad to the screen for visualization.
    index_buffer: wgpu::Buffer,
    /// Render pipeline for drawing a quad to the screen for visualization.
    render_pipeline: wgpu::RenderPipeline,
}

impl DepthPass {
    pub const DEPTH_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    /// Create a new depth pass. Only one instance is needed per renderer.
    pub fn new(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let (depth_texture, depth_texture_view, depth_sampler) =
            Self::create_depth_texture(device, surface_config);

        // This bind group is used to render the depth buffer to the screen for
        // visualization. It only requirs the texture view and sampler, no other
        // uniforms are needed (e.g., view transform).
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("depth pass layout"),
            entries: &[
                // Slot 0: depth buffer texture view.
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    visibility: wgpu::ShaderStages::FRAGMENT,
                },
                // Slot 1: depth buffer texture sampler.
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    count: None,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    visibility: wgpu::ShaderStages::FRAGMENT,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("depth pass bind group"),
            layout: &bind_group_layout,
            entries: &[
                // Slot 0: depth buffer texture view.
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&depth_texture_view),
                },
                // Slot 1: depth buffer texture sampler.
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&depth_sampler),
                },
            ],
        });

        // Create a unique vertex and index buffer for a full screen quad that
        // will render the depth pass (if visualization is requested).
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("depth buffer quad vertex buffer"),
            contents: bytemuck::cast_slice(QUAD_VERTS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("depth buffer quad index buffer"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Load the depth shader which renders to the debug quad.
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("depth display shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../assets/depth_buffer.wgsl").into()),
        });

        // Create the render pipeline which is used for rendering the depth pass
        // for debugging or instructional purposes.
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("depth pass render pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("depth pass pipeline layout"),
                    bind_group_layouts: &[&bind_group_layout],
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
            depth_texture,
            depth_texture_view,
            depth_sampler,
            bind_group_layout,
            bind_group,
            vertex_buffer,
            index_buffer,
            render_pipeline,
        }
    }

    /// Get the depth texture view which is required for writing to the depth
    /// buffer or reading it.
    pub fn depth_texture_view(&self) -> &wgpu::TextureView {
        &self.depth_texture_view
    }

    /// Resize the depth buffer to match the new window size. This must be called
    /// when the window is resized and only after `surface_config` is resized.
    pub fn resize(&mut self, device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration) {
        // Recreate the depth buffer texture, view and sampler when resized.
        // TODO: Is there any way to re-use these existing resources?
        let (depth_texture, depth_texture_view, depth_sampler) =
            Self::create_depth_texture(device, surface_config);

        self.depth_texture = depth_texture;
        self.depth_texture_view = depth_texture_view;
        self.depth_sampler = depth_sampler;

        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("depth pass bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                // Slot 0: depth buffer texture view.
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.depth_texture_view),
                },
                // Slot 1: depth buffer texture sampler.
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.depth_sampler),
                },
            ],
        });
    }

    /// Draw the contents of the depth buffer to the screen for visualization
    /// purposes.
    pub fn draw(
        &self,
        output_view: &wgpu::TextureView,
        command_encoder: &mut wgpu::CommandEncoder,
    ) {
        let mut depth_render_pass =
            command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("depth buffer visualization render pass"),
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

        depth_render_pass.set_pipeline(&self.render_pipeline);
        depth_render_pass.set_bind_group(0, &self.bind_group, &[]);
        depth_render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        depth_render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        depth_render_pass.draw_indexed(0..QUAD_INDICES.len() as u32, 0, 0..1);
    }

    /// Helper method that creates the depth texture as well as its associated
    /// view and sampler objects.
    fn create_depth_texture(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
        // Create the GPU backing texture for the depth buffer. Including
        // `TextureUsages::RENDER_ATTACHMENT` in the usage flags ensures depth
        // information can be written to this texture.
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth buffer texture"),
            size: wgpu::Extent3d {
                width: surface_config.width.max(1),
                height: surface_config.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_TEXTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[Self::DEPTH_TEXTURE_FORMAT],
        });

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // The sampler for this depth texture can optionally be used for
        // visualizing the depth buffer.
        let depth_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: None,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        (depth_texture, depth_texture_view, depth_sampler)
    }
}
