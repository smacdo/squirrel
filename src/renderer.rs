mod instancing;
mod meshes;
mod shaders;
mod textures;

use std::time::Duration;

use glam::{Quat, Vec3};
use instancing::{spawn_object_instances_as_grid, ModelInstanceBuffer};
use tracing::{info, warn};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::camera::Camera;
use crate::gameplay::CameraController;

use shaders::{PerFrameUniforms, PerModelUniforms};
use textures::Texture;

/// The renderer is pretty much everything right now while I ramp up on the
/// wgpu to get a basic 2d/3d prototype up.
pub struct Renderer<'a> {
    // TODO(scott): These should not be public, make methods on renderer.
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    pub window_size: winit::dpi::PhysicalSize<u32>,
    pub depth_texture: Texture,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: usize,
    pub camera: Camera,
    pub per_frame_uniforms: PerFrameUniforms,
    pub model_uniforms: PerModelUniforms,
    /// XXX(scott): This is a temporary demonstration of swapping textures.
    pub model_uniforms_2: PerModelUniforms,
    pub switch_to_uniform_2: bool,
    pub model_instance_buffer: ModelInstanceBuffer,

    // TODO(scott): extract gameplay code into separate module.
    pub camera_controller: CameraController,
    sys_time_elapsed: std::time::Duration,

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

        // Create the per-model bind group layout.
        // Inputs:
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

        // Initialize a default camera.
        // Position it one unit up, and two units back from world origin and
        // have it look at the origin.
        // +y is up
        // +z is out of the screen.
        let camera = Camera::new(
            //Vec3::new(0.0, 0.0, 3.0),
            Vec3::new(0.0, 5.0, 10.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            f32::to_radians(45.0),
            0.1,
            100.0,
            surface_config.width,
            surface_config.height,
        );

        // Create a uniform per-frame buffer to store shader values such as
        // the camera projection matrix.
        let per_frame_uniforms = PerFrameUniforms::new(&device);

        // Load the default shader.
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Create a depth buffer to ensure fragments are correctly rendered
        // back to front.
        let depth_texture =
            Texture::create_depth_texture(&device, &surface_config, Some("depth buffer"));

        // Create the default render pipeline layout and render pipeline objects.
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    per_frame_uniforms.bind_group_layout(),
                    &per_model_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[shaders::Vertex::desc(), ModelInstanceBuffer::layout_desc()],
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_TEXTURE_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // Fragments drawn front to back.
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
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

        // Load textures and other values needed by the model when rendering.
        let model_uniforms = PerModelUniforms::new(
            &device,
            &per_model_bind_group_layout,
            Texture::from_image_bytes(
                &device,
                &queue,
                include_bytes!("assets/test.png"),
                Some("diffuse texture"),
            )
            .unwrap(),
        );

        let model_uniforms_2 = PerModelUniforms::new(
            &device,
            &per_model_bind_group_layout,
            Texture::from_image_bytes(
                &device,
                &queue,
                include_bytes!("assets/null.png"),
                Some("diffuse texture"),
            )
            .unwrap(),
        );

        // Instancing demo!
        let model_instance_buffer = ModelInstanceBuffer::new(
            &device,
            spawn_object_instances_as_grid(10, 10, Vec3::new(5.0, 0.0, 5.0), 45_f32.to_radians()),
        );

        // TODO: Log info like GPU name, etc after creation.

        // Initialization (hopefully) complete!
        Self {
            surface,
            device,
            queue,
            surface_config,
            window_size,
            depth_texture,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            model_uniforms,
            model_uniforms_2,
            switch_to_uniform_2: false,
            model_instance_buffer,
            camera,
            sys_time_elapsed: Default::default(),
            per_frame_uniforms,
            camera_controller: CameraController::new(),
            window,
        }
    }

    pub fn window(&self) -> &Window {
        self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // TODO(scott); Ensure resize doesn't fire nonstop when drag-resizing.
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

            // Recreate the depth buffer to match the new window size.
            self.depth_texture = Texture::create_depth_texture(
                &self.device,
                &self.surface_config,
                Some("depth buffer"),
            );

            // Recreate the camera viewport to match the new window size.
            self.camera
                .set_viewport_size(new_size.width, new_size.height)
                .unwrap_or_else(|e| warn!("{e}"))
        }
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        // XXX(scott): Quick hack to swap demonstrate texture swapping.
        if let winit::event::WindowEvent::KeyboardInput {
            event:
                winit::event::KeyEvent {
                    physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyT),
                    state: winit::event::ElementState::Pressed,
                    ..
                },
            ..
        } = event
        {
            self.switch_to_uniform_2 = !self.switch_to_uniform_2;
        }

        self.camera_controller.process_input(event)
    }

    // TODO(scott): update should get a delta time, and pass the delta time to
    // the camera controller.
    pub fn update(&mut self, delta: Duration) {
        // Allow camera controoler to control the scene's camera.
        self.camera_controller
            .update_camera(&mut self.camera, delta);

        // Update per-frame shader uniforms.
        self.sys_time_elapsed += delta;

        self.per_frame_uniforms
            .set_view_projection(self.camera.view_projection_matrix());
        self.per_frame_uniforms
            .set_time_elapsed_seconds(self.sys_time_elapsed);

        self.per_frame_uniforms.write_to_gpu(&self.queue);

        // Rotate all model instances to demonstrate dynamic updates.
        let angle = (self.sys_time_elapsed.as_secs_f32() * 25.0).rem_euclid(365.0);
        self.model_instance_buffer
            .instances_mut()
            .iter_mut()
            .for_each(|x| {
                x.rotation = if x.position == Vec3::ZERO {
                    Quat::from_axis_angle(Vec3::Z, 0.0)
                } else {
                    Quat::from_axis_angle(x.position.normalize(), angle.to_radians())
                };
                x.rotation = Quat::IDENTITY;
            });

        self.model_instance_buffer.write_to_gpu(&self.queue);
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

        // Draw a simple triangle.
        {
            // Prepare the default rendering pipeline for drawing a mesh.

            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Clear the back buffer when rendering.
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        // Write the values from the fragment shader to the back
                        // buffer.
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, self.per_frame_uniforms.bind_group(), &[]);
            render_pass.set_bind_group(
                1,
                if self.switch_to_uniform_2 {
                    self.model_uniforms_2.bind_group()
                } else {
                    self.model_uniforms.bind_group()
                },
                &[],
            );

            // Bind the mesh's vertex and index buffers.
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.model_instance_buffer.gpu_buffer().slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Draw the mesh.
            //render_pass.draw_indexed(0..self.num_indices as u32, 0, 0..1);
            render_pass.draw_indexed(
                0..self.num_indices as u32,
                0,
                0..self.model_instance_buffer.instances().len() as _,
            );
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
