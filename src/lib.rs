#[cfg(target_arch = "wasm32")]
mod wasm_support;

mod camera;
mod content;
mod game_app;
mod gameplay;
mod math_utils;
mod platform;
mod renderer;

use game_app::multi_cube_demo::MultiCubeDemo;
use game_app::GameAppHost;
use platform::SystemTime;
use renderer::Renderer;
use tracing::info;
use tracing_log::log::{self};
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run_main() {
    // Initialize logging before doing anything else.
    // TODO: Configure tracing to emit INFO+ for wgpu, and DEBUG+ for squirrel
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            wasm_support::logging_init();
        } else {
            let stdout_subscriber = tracing_subscriber::fmt().pretty().finish();
            tracing::subscriber::set_global_default(stdout_subscriber)
                .expect("failed to install stdout global tracing subscriber");
        }
    }

    tracing_log::LogTracer::init().expect("failed to initialize LogTracer");

    info!(
        "demo data: {}",
        platform::load_as_string("demo_cube.mtl").await.unwrap()
    );

    // Create main window for rendering.
    log::info!("initializing event loop and creating a main window");

    let event_loop = EventLoop::new().expect("failed to create main window event loop");
    let main_window = WindowBuilder::new()
        .with_title("Squirrel Render Window")
        .build(&event_loop)
        .unwrap();

    // Create a canvas for rendering (for webasm mode).
    #[cfg(target_arch = "wasm32")]
    wasm_support::create_canvas(&main_window);

    // Initialize the renderer.
    log::info!("creating render window");

    let mut renderer = Renderer::new(&main_window).await;
    let game = MultiCubeDemo::new(&mut renderer);

    let mut game_host = GameAppHost::new(renderer, Box::new(game));

    // Main window event loop.
    //
    // NOTE: Window events are first sent to a custom input processer, and only
    //       if the processor returns false are they further dispatched in the
    //       event dispatcher below.
    log::info!("starting main window event loop");
    let mut last_redraw = SystemTime::now();

    let mut surface_configured = false;

    // EXPERIMENT: Recreate and upload the texture after resume event fires.
    let _recreate_texture_once = false;

    event_loop
        .run(move |event, control_flow| {
            let renderer_window_id = game_host.renderer().window().id();

            match event {
                Event::Resumed => {
                    info!("resumed event received, rendering can start");
                    surface_configured = true;
                }
                Event::WindowEvent { event, window_id } if window_id == renderer_window_id => {
                    // Allow the renderer to consume input events prior to processing them here.
                    if game_host.input(&event) {
                        // Event processed by renderer, do not continue.
                    } else {
                        // Event not processed by renderer, handle it here.
                        match event {
                            // Redraw window:
                            WindowEvent::RedrawRequested => {
                                // Request a redraw.
                                // TODO(scott): Switch to continuous event loop.
                                game_host.renderer().window.request_redraw();

                                // Measure amount of time elapsed.
                                let time_since_last_redraw = SystemTime::now() - last_redraw;
                                last_redraw = SystemTime::now();

                                // Don't try rendering until the window surface
                                // is ready.
                                if !surface_configured {
                                    return;
                                }

                                // Update simulation state and then render.
                                // TODO: Fixed step updates with render logic.
                                game_host.update_sim(time_since_last_redraw);
                                game_host.render(time_since_last_redraw);
                            }
                            // Window close requested:
                            WindowEvent::CloseRequested => control_flow.exit(),
                            // Keyboard input:
                            WindowEvent::KeyboardInput { event, .. } => {
                                // Quit when escape is pressed.
                                if event.state == ElementState::Pressed {
                                    if let Key::Named(NamedKey::Escape) = event.logical_key {
                                        control_flow.exit()
                                    }
                                }
                            }
                            // Window resized:
                            WindowEvent::Resized(physical_size) => {
                                game_host.window_resized(physical_size.width, physical_size.height)
                            }
                            // Window DPI changed:
                            WindowEvent::ScaleFactorChanged { .. } => {
                                game_host.scale_factor_changed()
                            }
                            _ => {}
                        }
                    }
                }
                Event::DeviceEvent {
                    device_id: _device_id,
                    event: device_event,
                } => match device_event {
                    DeviceEvent::MouseMotion { delta } => game_host.mouse_motion(delta.0, delta.1),
                    DeviceEvent::MouseWheel {
                        delta: MouseScrollDelta::LineDelta(delta_x, delta_y),
                    } => game_host.mouse_scroll_wheel(delta_x as f64, delta_y as f64),
                    _ => {}
                },

                _ => {}
            }
        })
        .expect("main window event loop processing failed");

    // All done.
    log::info!("exiting main window loop");
}
