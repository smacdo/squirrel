
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

/// Responsible for receiving input events and rendering output to a window.
///
/// NOTE: The `window` field must come after the `surface` field because the
///       destruction order is important as `surface` has unsafe references to
///       the window.
struct RendererWindow {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    //window: winit::window::Window,
}

impl RendererWindow {
    async fn new(window: Window) -> Self {
        // Fetch a copy of the backend instance, which could be any backend
        // renderer such as Vulkan, Metal, DX12, WebGPU...
        let instance = wgpu::Instance::default();

        /*
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        */

        // Create a hardware surface unique to the selected backend as the
        // render target. The surface must live as long as the window and since
        // `Self` owns the window this should be safe.
        /*
                let surface = unsafe { instance.create_surface(&window) }
                    .expect("failed to create renderer window surface");
        */
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        todo!()
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalPosition<u32>) {
        todo!("implement me! -- renderer_window.rs:27");
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        todo!("implement me! -- renderer_window.rs:31");
    }

    fn update(&mut self) {
        todo!("implement me! -- renderer_window.rs:35");
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        todo!("implement me! -- renderer_window.rs:39");
    }
}
