mod debug;
mod gpu_buffers;
mod instancing;
mod meshes;
mod models;
mod passes;
mod shaders;
mod shading;
mod textures;

use std::rc::Rc;
use std::time::Duration;

use debug::DebugState;
use glam::{Quat, Vec2, Vec3};
use gpu_buffers::{DynamicGpuBuffer, UniformBindGroup};
use meshes::{builtin_mesh, BuiltinMesh};
use models::{DrawModel, Mesh, Model, Submesh};
use shaders::{BindGroupLayouts, PerFrameShaderVals};
use shading::{DirectionalLight, LightAttenuation, Material, PointLight, SpotLight};
use tracing::{info, warn};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::gameplay::ArcballCameraController;
use crate::math_utils::rotate_around_pivot;
use crate::{camera::Camera, gameplay::CameraController};

const INITIAL_CUBE_POS: &[Vec3] = &[
    Vec3::new(0.0, 0.0, 0.0),
    Vec3::new(2.0, 5.0, -15.0),
    Vec3::new(-1.5, -2.2, -2.5),
    Vec3::new(-3.8, -2.0, -12.3),
    Vec3::new(2.4, -0.4, -3.5),
    Vec3::new(-1.7, 3.0, 7.5),
    Vec3::new(1.3, -2.0, -2.5),
    Vec3::new(1.5, 2.0, -2.5),
    Vec3::new(1.5, 0.2, -1.5),
    Vec3::new(-1.3, 1.0, -1.5),
];

/// The renderer is pretty much everything right now while I ramp up on the
/// wgpu to get a basic 2d/3d prototype up.
pub struct Renderer<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    window_size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    per_frame_uniforms: PerFrameShaderVals,
    depth_pass: passes::DepthPass,
    light_debug_pass: passes::LightDebugPass,
    sys_time_elapsed: std::time::Duration,
    debug_state: DebugState,
    // TODO(scott): extract gameplay code into separate module.
    pub camera: Camera,
    pub camera_controller: ArcballCameraController,
    point_light: PointLight,
    directional_light: DirectionalLight,
    spot_light: SpotLight,
    models: Vec<Model>,
    pub time_to_update: f32,
    // XXX(scott): `window` must be the last field in the struct because it needs
    // to be dropped after `surface`, because the surface contains unsafe
    // references to `window`.
    pub window: &'a Window,
}

// TODO: Renderer::new() should return Result<Self> and remove .unwrap().

impl<'a> Renderer<'a> {
    const STANDARD_SHADER: &'static str = include_str!("standard_shader.wgsl");
    const CAMERA_POS: Vec3 = Vec3::new(1.5, 1.0, 5.0);
    const CAMERA_LOOK_AT: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    const POINT_LIGHT: PointLight = PointLight {
        position: Vec3::new(1.2, 1.0, 2.0),
        attenuation: LightAttenuation {
            constant: 1.0,
            linear: 0.09,
            quadratic: 0.032,
        },
        color: Vec3::new(0.8, 0.8, 0.8),
        ambient: 0.0425,
        specular: 1.0,
    };
    const DIRECTIONAL_LIGHT: DirectionalLight = DirectionalLight {
        direction: Vec3::new(0.0, -1.0, 0.0),
        color: Vec3::new(0.3, 0.3, 0.3),
        ambient: 0.01,
        specular: 1.0,
    };
    const SPOT_LIGHT: SpotLight = SpotLight {
        position: Vec3::ZERO,
        direction: Vec3::ZERO,
        cutoff_radians: 0.2181662,       // 12.5 degree.
        outer_cutoff_radians: 0.3054326, // 17.5 degree.
        color: Vec3::new(0.8, 0.8, 0.8),
        attenuation: LightAttenuation {
            constant: 1.0,
            linear: 0.09,
            quadratic: 0.032,
        },
        ambient: 0.01,
        specular: 1.0,
    };

    pub async fn new(window: &'a Window) -> Self {
        let window_size = window.inner_size();
        info!("initial renderer size: {:?}", window_size);

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

        // Create the registry of common bind group layouts that must be reused
        // each time an instance of that bind group is created.
        let bind_group_layouts = BindGroupLayouts::new(&device);

        // Initialize a default camera.
        // Position it one unit up, and two units back from world origin and
        // have it look at the origin.
        // +y is up
        // +z is out of the screen.
        let camera = Camera::new(
            Self::CAMERA_POS,
            Self::CAMERA_LOOK_AT,
            Vec3::new(0.0, 1.0, 0.0),
            f32::to_radians(45.0),
            0.1,
            100.0,
            surface_config.width,
            surface_config.height,
        );

        // Create a uniform per-frame buffer to store shader values such as
        // the camera projection matrix.
        let mut per_frame_uniforms = PerFrameShaderVals::new(&device, &bind_group_layouts);
        per_frame_uniforms.set_output_is_srgb(surface_format.is_srgb());

        // Load the default shader.
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(Self::STANDARD_SHADER.into()),
        });

        // Create the default render pipeline layout and render pipeline objects.
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &bind_group_layouts.per_frame_layout,
                    &bind_group_layouts.per_model_layout,
                    &bind_group_layouts.per_submesh_layout,
                ],
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: passes::DepthPass::DEPTH_TEXTURE_FORMAT,
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

        // Generate a cube mesh and then spawn multiple instances of it for rendering.
        let (vertices, indices) = builtin_mesh(BuiltinMesh::Cube);

        let cube_mesh = Rc::new(Mesh::new(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cube Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }),
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cube Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            }),
            indices.len() as u32,
            vec![Submesh::new(
                &device,
                &bind_group_layouts,
                0..indices.len() as u32,
                0,
                &Material {
                    ambient_color: Vec3::new(1.0, 1.0, 1.0),
                    diffuse_color: Vec3::new(1.0, 1.0, 1.0),
                    diffuse_map: Rc::new(
                        textures::from_image_bytes(
                            &device,
                            &queue,
                            include_bytes!("assets/crate_diffuse.dds"),
                            Some("crate diffuse texture"),
                        )
                        .unwrap(),
                    ),
                    specular_color: Vec3::new(1.0, 1.0, 1.0),
                    specular_map: Rc::new(
                        textures::from_image_bytes(
                            &device,
                            &queue,
                            include_bytes!("assets/crate_specular.dds"),
                            Some("crate specular texture"),
                        )
                        .unwrap(),
                    ),
                    specular_power: 64.0,
                    emissive_map: Rc::new(textures::new_1x1(
                        &device,
                        &queue,
                        [0, 0, 0],
                        Some("default emission texture map"),
                    )),
                },
            )],
        ));

        // Set up scene.
        let mut models: Vec<Model> = Vec::with_capacity(INITIAL_CUBE_POS.len());

        for initial_pos in INITIAL_CUBE_POS {
            models.push(Model::new(
                &device,
                &bind_group_layouts,
                *initial_pos,
                Quat::IDENTITY,
                Vec3::ONE,
                cube_mesh.clone(),
            ));
        }

        // Set up additional render passes.
        let depth_pass = passes::DepthPass::new(&device, &surface_config);
        let light_debug_pass =
            passes::LightDebugPass::new(&device, &surface_config, &bind_group_layouts);

        // Initialization (hopefully) complete!
        Self {
            surface,
            device,
            queue,
            surface_config,
            window_size,
            render_pipeline,
            point_light: Self::POINT_LIGHT,
            directional_light: Self::DIRECTIONAL_LIGHT,
            spot_light: Self::SPOT_LIGHT,
            models,
            camera,
            sys_time_elapsed: Default::default(),
            per_frame_uniforms,
            camera_controller: ArcballCameraController::new(),
            depth_pass,
            light_debug_pass,
            debug_state: Default::default(),
            window,
            time_to_update: 0.0,
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
            self.depth_pass.resize(&self.device, &self.surface_config);

            // Recreate the camera viewport to match the new window size.
            self.camera
                .set_viewport_size(new_size.width, new_size.height)
                .unwrap_or_else(|e| warn!("{e}"))
        }
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.debug_state.process_input(event);
        self.camera_controller.process_input(event)
    }

    pub fn update(&mut self, delta: Duration) {
        // Allow camera controoler to control the scene's camera.
        self.camera_controller
            .update_camera(&mut self.camera, delta);

        // Spot light follows the camera.
        self.spot_light.position = self.camera.eye();
        self.spot_light.direction = self.camera.forward();

        // Update per-frame shader uniforms.
        self.sys_time_elapsed += delta;

        self.per_frame_uniforms
            .set_view_projection(self.camera.view_projection_matrix());
        self.per_frame_uniforms.set_view_pos(self.camera.eye());
        self.per_frame_uniforms
            .set_directional_light(&self.directional_light);
        self.per_frame_uniforms.set_spot_light(&self.spot_light);
        self.per_frame_uniforms
            .set_time_elapsed_seconds(self.sys_time_elapsed);

        self.per_frame_uniforms.update_gpu(&self.queue);

        // Make the light orbit around the scene.
        let sys_time_secs: f32 = self.sys_time_elapsed.as_secs_f32();

        let light_xy = rotate_around_pivot(
            Vec2::new(0.0, 0.0),
            1.0,
            (sys_time_secs * 24.0).to_radians(),
        );

        self.point_light.position = Vec3::new(light_xy.x, light_xy.y, light_xy.y);

        // Update uniforms for each model that will be rendered.
        for model in &mut self.models.iter_mut() {
            model.uniforms_mut().set_point_light(&self.point_light);
            model.prepare(&self.queue);
        }

        // Passes / overlays.
        self.light_debug_pass.add_point_light(&self.point_light);
        self.light_debug_pass.prepare(&self.queue);
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

        // Draw all models in the scene.
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Clear the back buffer when rendering.
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        // Write the values from the fragment shader to the back
                        // buffer.
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth_pass.depth_texture_view(),
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

            debug_assert!(!self.per_frame_uniforms.is_dirty());
            render_pass.set_bind_group(0, self.per_frame_uniforms.bind_group(), &[]);

            for model in self.models.iter() {
                render_pass.draw_model(model);
            }
        }

        // Debug pass visualization.
        self.light_debug_pass.draw(
            &view,
            self.depth_pass.depth_texture_view(),
            &self.per_frame_uniforms,
            &mut command_encoder,
        );

        // Depth pass visualization.
        if self.debug_state.visualize_depth_pass {
            self.depth_pass.draw(&view, &mut command_encoder);
        }

        // All done - submit commands for execution.
        self.queue.submit(std::iter::once(command_encoder.finish()));
        backbuffer.present();

        self.light_debug_pass.finish_frame();

        Ok(())
    }

    pub fn window_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window_size
    }
}
