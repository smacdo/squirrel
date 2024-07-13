mod debug;
mod gpu_buffers;
mod instancing;
pub mod lighting;
pub mod materials;
pub mod meshes;
pub mod models;
mod passes;
pub mod scene;
pub mod shaders;
pub mod textures;

use std::{rc::Rc, time::Duration};

use debug::DebugState;
use glam::{Mat4, Quat, Vec3};
use gpu_buffers::{DynamicGpuBuffer, UniformBindGroup};
use models::{DrawModel, Mesh, Model};
use scene::Scene;
use shaders::{lit_shader, BindGroupLayouts, PerFrameShaderVals, PerModelShaderVals, VertexLayout};
use slotmap::{new_key_type, SlotMap};
use tracing::{info, warn};
use winit::window::Window;

use crate::{camera::Camera, content::DefaultTextures};

// TODO: Need to move wgpu device, queue and other values out of the renderer
//       to allow for code to create and update GPU resources w/out reading pub
//       properties from the renderer instance. This is especially needed to
//       split content loading away from the renderer.
//
//       This strongly affects how GameApp::load_content(...) works!

// TODO: Move camera out of the renderer.

// TODO: Consider moving things like camera, lights, models to a scene container.
// Doesn't have to be anything fancy since I'm not sure where all of this info
// should live yet, eg does renderer own the scene or the game?

// TODO: Remove pub access to renderer props like device, queue, bind group etc.
// I'm deferring these decisions right now because I think this will need a lot
// of working and involve figuring out how to do asset loading and shader swaps.

// TODO: Renderer::new() should return Result<Self> and remove .unwrap().

new_key_type! { pub struct ModelShaderValsKey; }

/// The renderer is pretty much everything right now while I ramp up on WGPU
/// and other graphics tutorials to get a basic 2d/3d prototype up.
pub struct Renderer<'a> {
    surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub default_textures: DefaultTextures,
    pub bind_group_layouts: BindGroupLayouts,
    surface_config: wgpu::SurfaceConfiguration,
    window_size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    per_frame_uniforms: PerFrameShaderVals,
    depth_pass: passes::DepthPass,
    light_debug_pass: passes::LightDebugPass,
    sys_time_elapsed: std::time::Duration,
    debug_state: DebugState,
    pub camera: Camera,
    pub model_shader_vals: SlotMap<ModelShaderValsKey, PerModelShaderVals>,
    // XXX(scott): `window` must be the last field in the struct because it needs
    // to be dropped after `surface`, because the surface contains unsafe
    // references to `window`.
    pub window: &'a Window,
}

impl<'a> Renderer<'a> {
    const CAMERA_POS: Vec3 = Vec3::new(1.5, 1.0, 5.0);
    const CAMERA_LOOK_AT: Vec3 = Vec3::new(0.0, 0.0, 0.0);

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

        // Load the default shader and associated resources.
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(lit_shader::SHADER_CODE.into()),
        });

        let default_textures = DefaultTextures::new(&device, &queue);

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
                buffers: &[models::Vertex::vertex_buffer_layout()],
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

        // Set up additional render passes.
        let depth_pass = passes::DepthPass::new(&device, &surface_config);
        let light_debug_pass =
            passes::LightDebugPass::new(&device, &surface_config, &bind_group_layouts);

        // Initialization (hopefully) complete!
        Self {
            surface,
            device,
            queue,
            default_textures,
            bind_group_layouts,
            surface_config,
            window_size,
            render_pipeline,
            camera,
            model_shader_vals: SlotMap::with_key(),
            sys_time_elapsed: Default::default(),
            per_frame_uniforms,
            depth_pass,
            light_debug_pass,
            debug_state: Default::default(),
            window,
        }
    }

    pub fn window(&self) -> &Window {
        self.window
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        // TODO(scott); Ensure resize doesn't fire nonstop when drag-resizing.
        if new_width == 0 || new_height == 0 {
            warn!(
                "invalid width of {} or height {} when resizing",
                new_width, new_height
            );
        } else {
            self.window_size = winit::dpi::PhysicalSize::new(new_width, new_height);
            self.surface_config.width = new_width;
            self.surface_config.height = new_height;
            self.surface.configure(&self.device, &self.surface_config);

            // Recreate the depth buffer to match the new window size.
            self.depth_pass.resize(&self.device, &self.surface_config);

            // Recreate the camera viewport to match the new window size.
            self.camera
                .set_viewport_size(new_width, new_height)
                .unwrap_or_else(|e| warn!("{e}"))
        }
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) {
        self.debug_state.process_input(event);
    }

    fn prepare_render(&mut self, scene: &Scene, delta: Duration) {
        // Update renderer per-frame shader uniforms.
        self.sys_time_elapsed += delta;
        self.per_frame_uniforms
            .set_time_elapsed_seconds(self.sys_time_elapsed);

        self.per_frame_uniforms
            .set_view_projection(self.camera.view_projection_matrix());
        self.per_frame_uniforms.set_view_pos(self.camera.eye());

        // Update renderer per-scene shader uniforms.
        self.per_frame_uniforms.clear_lights();

        for light in &scene.directional_lights {
            self.per_frame_uniforms.add_directional_light(light);
        }

        for light in &scene.spot_lights {
            self.per_frame_uniforms.add_spot_light(light);
        }

        // Update uniforms for each model that will be rendered.
        for model in scene.models.iter() {
            let model_sv = &mut self.model_shader_vals[model.model_sv_key];

            // Does the transform matrix need to be updated?
            if model.is_model_sv_dirty() {
                model_sv.set_local_to_world(Mat4::from_scale_rotation_translation(
                    model.scale(),
                    model.rotation(),
                    model.translation(),
                ));
            }

            // Add lights closest to the model.
            model_sv.clear_lights();

            for light in &scene.point_lights {
                model_sv.add_point_light(light);
            }

            // Copy the model's shader values to the GPU and then mark its
            // shader values object as having been updated.
            model_sv.update_gpu(&self.queue);
            model.mark_model_sv_updated();
        }

        // Let render overlays update resources.
        self.light_debug_pass.prepare(&self.queue, scene);

        // Copy updated per frame uniform values to the GPU.
        self.per_frame_uniforms.update_gpu(&self.queue);
    }

    pub fn render(&mut self, scene: &Scene, delta: Duration) -> Result<(), wgpu::SurfaceError> {
        // Prepare GPU resources for rendering.
        self.prepare_render(scene, delta);

        // Start rendering the frame.
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

            for model in scene.models.iter() {
                render_pass.draw_model(model, &self.model_shader_vals[model.model_sv_key]);
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

    /// Returns a new model that can be added to a scene and rendered.
    pub fn create_model(
        &mut self,
        mesh: Rc<Mesh>,
        translation: Vec3,
        rotation: Quat,
        scale: Vec3,
    ) -> Model {
        Model::new(
            self.model_shader_vals.insert(PerModelShaderVals::new(
                &self.device,
                &self.bind_group_layouts,
            )),
            mesh,
            translation,
            rotation,
            scale,
        )
    }
}
