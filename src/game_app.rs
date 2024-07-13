pub mod multi_cube_demo;

use std::time::Duration;

use tracing::{debug, error, warn};

use crate::renderer::{scene::Scene, Renderer};

/// Dispatches events coming from the underlying platform to the game for
/// execution.
pub struct GameAppHost<'a> {
    renderer: Renderer<'a>, // TODO: Refactor so renderer does not need to be stored.
    game: Box<dyn GameApp>,
    mouse_captured: bool,
}

impl<'a> GameAppHost<'a> {
    pub fn new(renderer: Renderer<'a>, game: Box<dyn GameApp>) -> Self {
        Self {
            renderer,
            game,
            mouse_captured: false,
        }
    }

    pub fn load_content(&mut self) -> anyhow::Result<()> {
        self.game.load_content(&mut self.renderer)
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        // TODO: Resolve that self.renderer.input is ()
        // If renderer.input returns false do not let game app handle input but
        // also issue a warning that it was overridden?
        self.renderer.input(event);
        self.game.input(event)
    }

    pub fn update_sim(&mut self, delta: Duration) {
        self.game.update_sim(delta)
    }

    pub fn render(&mut self, delta: Duration) {
        self.game.prepare_render(&mut self.renderer, delta);

        match self.renderer.render(self.game.render_scene(), delta) {
            Ok(_) => {}
            // Reconfigure surface when lost:
            Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) => {
                warn!("handling surface lost or outdated event by re-applying current window size");
                let window_size = self.renderer.window_size();
                self.renderer.resize(window_size.width, window_size.height);
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

    /// Handles when the game window ("rendering window") is resized.
    pub fn window_resized(&mut self, new_width: u32, new_height: u32) {
        self.renderer.resize(new_width, new_height)
    }

    /// Handles when Windows DPI scaling is changed.
    pub fn scale_factor_changed(&mut self) {
        let new_size = self.renderer.window().inner_size();
        self.renderer.resize(new_size.width, new_size.height)
    }

    /// Handles when the mouse moves.
    pub fn mouse_motion(&mut self, delta_x: f64, delta_y: f64) {
        self.game.mouse_motion(delta_x, delta_y)
    }

    /// Handles when the mouse wheel is scrolled up or down.
    pub fn mouse_scroll_wheel(&mut self, delta_x: f64, delta_y: f64) {
        self.game.mouse_scroll_wheel(delta_x, delta_y)
    }

    pub fn is_mouse_captured(&self) -> bool {
        self.mouse_captured
    }

    pub fn set_mouse_captured(&mut self, is_captured: bool) {
        let window = self.renderer.window();

        // Lock the cursor to the screen.
        if let Err(_e) = window.set_cursor_grab(if is_captured {
            winit::window::CursorGrabMode::Locked
        } else {
            winit::window::CursorGrabMode::None
        }) {
            // TODO: Handle error by manually locking the cursor?
            //       First check when this can fail.
            warn!("failed to lock/unlock cursor")
        };

        // The cursor should be hidden when the mouse is captured.
        window.set_cursor_visible(!is_captured);

        // Track the mouse capture state.
        debug!("mouse_captured = {is_captured}");
        self.mouse_captured = is_captured;
    }
}

/// A specific game or demo scene implementation.
pub trait GameApp {
    /// Loads content required by the game prior to the start of rendering
    fn load_content(&mut self, renderer: &mut Renderer) -> anyhow::Result<()>;

    /// Advances the game's simulation state by the given `delta`.
    fn update_sim(&mut self, delta: Duration);

    /// Prepares GPU resources for rendering in the upcoming frame.
    fn prepare_render(&mut self, renderer: &mut Renderer, delta: Duration);

    /// Called anytime there is a new input even from the host.
    fn input(&mut self, event: &winit::event::WindowEvent) -> bool;

    /// Called by the host when the user's mouse moves.
    fn mouse_motion(&mut self, _delta_x: f64, _delta_y: f64) {}

    /// Called by the host when user moves the scroll wheel up or down.
    fn mouse_scroll_wheel(&mut self, _delta_x: f64, _delta_y: f64) {}

    /// Returns the render scene for the game app.
    fn render_scene(&self) -> &Scene;
}
