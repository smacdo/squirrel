use glam::Vec3;
use tracing::{info, warn};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::camera::Camera;
use crate::meshes;
use crate::shaders::{self};
use crate::textures::Texture;

/// The renderer is pretty much everything right now while I ramp up on the
/// wgpu to get a basic 2d/3d prototype up.
pub struct Renderer<'a> {
    // TODO(scott): These should not be public, make methods on renderer.
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub window_size: winit::dpi::PhysicalSize<u32>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: usize,
    pub camera: Camera,
    pub camera_buffer: wgpu::Buffer,
    pub per_frame_bind_group: wgpu::BindGroup,
    pub per_model_bind_group: wgpu::BindGroup,
    pub texture: wgpu::Texture,
    /// XXX(scott): `window` must be the last field in the struct because it needs
    /// to be dropped after `surface`, because the surface contains unsafe
    /// references to `window`.
    pub window: &'a Window,
}

// TODO: Renderer::new() should return Result<Self> and remove .unwrap().

impl<'a> Renderer<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let window_size = window.inner_size();

        // Create a WGPU instance that can use any supported graphics API.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create the main rendering surface and then get an adapter that acts
        // as the handle to one of the machine's physical GPU(s).
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // Get a communication channel to the graphics card and a queue for
        // submitting commands to.
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        // Set the main rendering surface to use an sRGB texture, and then allow
        // all shaders to assume they are writing to an sRGB back buffer.
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        if surface_format.is_srgb() {
            info!("rendering surface supports sRGB");
        } else {
            info!("no sRGB support found for the main rendering surface, defaulting to first available");
        }

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        // Load a texture for rendering.
        let diffuse_bytes = include_bytes!("assets/wall.jpg");
        let texture =
            Texture::from_image_bytes(&device, &queue, diffuse_bytes, Some("diffuse texture"))
                .unwrap();

        // Create a bind group for texture(s) rendering.
        //  0 - diffuse texture
        //  1 - diffuse sampler
        let per_model_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("per-model bind group layout"),
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
            });

        let per_model_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("per-model bind group"),
            layout: &per_model_bind_group_layout,
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

        // Initialize a default camera.
        // Position it one unit up, and two units back from world origin and
        // have it look at the origin.
        // +y is up
        // +z is out of the screen.
        //
        // TODO: This doesn't work on webasm platforms because width/height
        //       isn't available until after renderer is initialized!
        let camera = Camera::new(
            Vec3::new(0.0, 0.0, 3.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            f32::to_radians(45.0),
            0.1,
            100.0,
            surface_config.width,
            surface_config.height,
        );
        let view_projection = camera.view_projection_matrix();

        // Create a uniform per-frame buffer to store shader values such as
        // the camera projection matrix.
        let per_frame_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("per-frame bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    // Camera.
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera buffer"),
            contents: bytemuck::cast_slice(&[view_projection]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let per_frame_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("per-frame bind group"),
            layout: &per_frame_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                // Camera
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Load the default shader.
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Create the default render pipeline layout and render pipeline objects.
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&per_frame_bind_group_layout, &per_model_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[shaders::Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create a vertex buffer and index for simple meshes.
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(meshes::RECT_VERTS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(meshes::RECT_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = meshes::RECT_INDICES.len();

        // TODO: Log info like GPU name, etc after creation.

        // Initialization (hopefully) complete!
        Self {
            surface,
            device,
            queue,
            surface_config,
            window_size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            per_model_bind_group,
            camera,
            camera_buffer,
            per_frame_bind_group,
            texture: texture.texture,
            window,
        }
    }

    pub fn window(&self) -> &Window {
        self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            warn!(
                "invalid width of {} or height {} when resizing",
                new_size.width, new_size.height
            );
        } else {
            self.window_size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);

            self.camera
                .set_viewport_size(new_size.width, new_size.height)
                .unwrap_or_else(|e| warn!("{e}"))
        }
    }

    pub fn input(&mut self, _event: &winit::event::WindowEvent) -> bool {
        // TODO(scott): implement me!
        false
    }

    pub fn update(&mut self) {
        // Copy camera projection matrix to shader.
        let view_projection = self.camera.view_projection_matrix();

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[view_projection]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let backbuffer = self.surface.get_current_texture()?;
        let view = backbuffer
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render loop encoder"),
                });

        // Clear back buffer.
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Draw a simple triangle.
            render_pass.set_pipeline(&self.render_pipeline);

            // Bind uniform buffers.
            render_pass.set_bind_group(0, &self.per_frame_bind_group, &[]);
            render_pass.set_bind_group(1, &self.per_model_bind_group, &[]);

            // Bind mesh buffers.
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Draw the mesh.
            render_pass.draw_indexed(0..self.num_indices as u32, 0, 0..1);
        }

        // All done - submit commands for execution.
        self.queue.submit(std::iter::once(command_encoder.finish()));
        backbuffer.present();

        Ok(())
    }

    pub fn window_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window_size
    }
}
