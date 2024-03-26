use tracing_log::{env_logger, log};
use tracing_subscriber::{filter, prelude::__tracing_subscriber_SubscriberExt, EnvFilter};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, ModifiersState, NamedKey},
    platform::modifier_supplement::KeyEventExtModifierSupplement,
    window::WindowBuilder,
};

fn main() {
    let stdout_subscriber = tracing_subscriber::fmt().pretty().finish();
    tracing::subscriber::set_global_default(stdout_subscriber)
        .expect("failed to install stdout global tracing subscriber");

    tracing_log::LogTracer::init().expect("failed to initialize LogTracer");

    pollster::block_on(run_main())
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    // TODO: Configure tracing to emit INFO+ for wgpu, and DEBUG+ for squirrel

    // Initialize the renderer.
    let mut renderer = Renderer::new(&main_window).await;

    // Main window event loop.
    //
    // NOTE: Window events are first sent to a custom input processer, and only
    //       if the processor returns false are they further dispatched in the
    //       event dispatcher below.
    log::info!("starting main window event loop");

    event_loop
        .run(move |event, control_flow| {
            let renderer_window_id = renderer.window().id();

            match event {
                Event::WindowEvent { event, window_id } if window_id == renderer_window_id => {
                    // Allow the renderer to consume input events prior to processing them here.
                    if renderer.input(&event) {
                        // Event processed by renderer, do not continue.
                    } else {
                        // Event not processed by renderer, handle it here.
                        match event {
                            // Redraw window:
                            WindowEvent::RedrawRequested => {
                                // Update simulation state.
                                renderer.update();

                                // Render simulation.
                                match renderer.render() {
                                    Ok(_) => {}
                                    // Reconfigure surface when lost:
                                    Err(wgpu::SurfaceError::Lost) => {
                                        renderer.resize(renderer.window_size())
                                    }
                                    // System is out of memory - bail out!
                                    Err(wgpu::SurfaceError::OutOfMemory) => {
                                        panic!("WGPU out of memory error")
                                    }
                                    // Other errors (outdated, timeout) should be resolved by next frame
                                    Err(e) => eprintln!("WGPU ERROR: {:?}", e),
                                }
                            }
                            // Window close requested:
                            WindowEvent::CloseRequested => control_flow.exit(),
                            // Keyboard input:
                            WindowEvent::KeyboardInput { event, .. } => {
                                if event.state == ElementState::Pressed {
                                    if let Key::Named(NamedKey::Escape) = event.logical_key {
                                        control_flow.exit()
                                    }
                                }
                            }
                            // Window resized:
                            WindowEvent::Resized(physical_size) => renderer.resize(physical_size),
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
    println!("Hello, world!");
}
