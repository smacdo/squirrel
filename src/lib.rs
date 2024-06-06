#[cfg(target_arch = "wasm32")]
mod wasm_support;

mod camera;
mod meshes;
mod shaders;
mod textures;
mod renderer;
mod gameplay;


use gameplay::CameraController;
use renderer::Renderer;
use tracing::{info, warn};
use tracing_log::log::{self, error};
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

    log::info!("initial renderer window size: {:?}", renderer.window_size);

    // Main window event loop.
    //
    // NOTE: Window events are first sent to a custom input processer, and only
    //       if the processor returns false are they further dispatched in the
    //       event dispatcher below.
    log::info!("starting main window event loop");
    let mut surface_configured = false;

    // EXPERIMENT: Recreate and upload the texture after resume event fires.
    let _recreate_texture_once = false;

    event_loop
        .run(move |event, control_flow| {
            let renderer_window_id = renderer.window().id();

            match event {
                Event::Resumed => {
                    info!("resumed event received, rendering can start");
                    surface_configured = true;
                }
                Event::WindowEvent { event, window_id } if window_id == renderer_window_id => {
                    // Allow the renderer to consume input events prior to processing them here.
                    if renderer.input(&event) {
                        // Event processed by renderer, do not continue.
                    } else {
                        // Event not processed by renderer, handle it here.
                        match event {
                            // Redraw window:
                            WindowEvent::RedrawRequested => {
                                // XXX(scott): is this needed? why?
                                renderer.window.request_redraw();

                                if !surface_configured {
                                    return;
                                }
                        

                                // Update simulation state.
                                renderer.update();

                                // Render simulation.
                                match renderer.render() {
                                    Ok(_) => {}
                                    // Reconfigure surface when lost:
                                    Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) => {
                                        warn!("handling surface lost or outdated event by re-applying current window size");
                                        renderer.resize(renderer.window_size())
                                    }
                                    // System is out of memory - bail out!
                                    Err(wgpu::SurfaceError::OutOfMemory) => {
                                        panic!("WGPU out of memory error")
                                    }
                                    // Other errors (outdated, timeout) should be resolved by next frame
                                    Err(e) => {
                                        error!("WGPU error, will skip frame and try to ignore: {e:?}");
                                    }
                                }
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
                                renderer.resize(physical_size)
                            }
                            // Window DPI changed:
                            WindowEvent::ScaleFactorChanged { .. } => {
                                // TODO(scott): The API diverges from the guide. Double check if correct.
                                let new_size = renderer.window().inner_size();
                                renderer.resize(new_size);
                            }
                            _ => {}
                        }
                    }
                }

                _ => {}
            }
        })
        .expect("main window event loop processing failed");

    // All done.
    log::info!("exiting main window loop");
}

