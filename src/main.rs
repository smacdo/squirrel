use tracing_log::{env_logger, log};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, ModifiersState, NamedKey},
    platform::modifier_supplement::KeyEventExtModifierSupplement,
    window::WindowBuilder,
};

fn main() {
    // Initialize logging.
    env_logger::init();

    // Create main window for rendering.
    log::info!("creating main window for rendering");

    let event_loop = EventLoop::new().expect("failed to create main window event loop");
    let main_window = WindowBuilder::new()
        .with_title("Squirrel Render Window")
        .build(&event_loop)
        .unwrap();

    // Main window event loop.
    log::info!("starting main window event loop");

    event_loop
        .run(move |event, control_flow| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => control_flow.exit(),
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == ElementState::Pressed {
                            if let Key::Named(NamedKey::Escape) = event.logical_key {
                                control_flow.exit()
                            }
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        //fill::fill_window(&main_window);
                    }
                    _ => {}
                }
            }
        })
        .expect("main window event loop processing failed");

    // All done.
    println!("Hello, world!");
}
