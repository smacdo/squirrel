use tracing_log::log;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

pub fn run_game_loop() {
    // Create main window for rendering.
    log::info!("creating main window for rendering");

    let event_loop = EventLoop::new().expect("failed to create main window event loop");
    let window_builder = WindowBuilder::new().with_title("Squirrel Render Window");

    #[cfg(wasm_platform)]
    let window_builder = {
        use winit::platform::web::WindowBuilderExtWebSys;
        window_builder.with_append(true)
    };

    let main_window = window_builder.build(&event_loop).unwrap();

    // Insert the main window into an HTML canvas element for WASM targets.
    #[cfg(wasm_platform)]
    wasm::insert_canvas(&main_window);

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

#[cfg(wasm_platform)]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn run() {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Warn).expect("failed to init console log");

        super::run_game_loop();
    }

    pub fn insert_canvas(window: &Window) -> web_sys::Element {
        use winit::platform::web::WindowExtWebSys;

        let canvas = window.canvas().unwrap();
        let mut surface = Surface::from_canvas(canvas.clone()).unwrap();
        surface
            .resize(
                NonZeroU32::new(canvas.width()).unwrap(),
                NonZeroU32::new(canvas.height()).unwrap(),
            )
            .unwrap();
        let mut buffer = surface.buffer_mut().unwrap();
        buffer.fill(0xFFF0000);
        buffer.present().unwrap();

        /*
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        let style = &canvas.style();
        style.set_property("margin", "50px").unwrap();
        */
    }
}
