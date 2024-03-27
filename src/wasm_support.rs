use console_error_panic_hook;
use tracing::info;
use tracing_wasm;
use winit::window::Window;

pub fn logging_init() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
}

pub fn create_canvas(window: &Window, width: u32, height: u32) {
    // Winit prevents sizing with CSS so the size has to be manually specified.
    info!("requesting canvas size of {width} x {height}");

    use winit::dpi::PhysicalSize;
    let _ = window.request_inner_size(PhysicalSize::new(width, height));

    use winit::platform::web::WindowExtWebSys;
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| {
            let element = d.get_element_by_id("wasm-container")?;
            let canvas = web_sys::Element::from(window.canvas().unwrap());
            element.append_child(&canvas).ok()?;
            Some(())
        })
        .expect("failed to append canvas to document body.");
}
