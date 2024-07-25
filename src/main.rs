use std::sync::{Arc, Mutex};
use winit::{
    dpi::PhysicalSize,
    event_loop::{self, EventLoop},
    window::Window,
};
mod model;
mod studio;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut game = GameEntry::Loading;
    let _ = event_loop.run_app(&mut game);
}

enum GameEntry {
    Loading,
    Ready(Game),
}

struct Game {
    window: Arc<Window>,
    context: Arc<Mutex<gfx::GfxContext>>,
    studio: Option<studio::Studio>,
}

mod gfx;
impl Game {
    fn bridge_with_gfx(&mut self, PhysicalSize::<u32> { width, height }: PhysicalSize<u32>) {
        let mut context = self.context.lock().unwrap();
        let mut surface_config = context
            .surface
            .get_default_config(&context.adapter, width, height)
            .unwrap();
        context.surface.configure(&context.device, &surface_config);
        let view_format = surface_config.format.add_srgb_suffix();
        surface_config.view_formats.push(view_format);
        context.surface_config = Some(surface_config);
    }
    fn list_painter(&mut self) {
        let context = self.context.clone();
        let mut studio_var = studio::Studio::new(context);

        studio_var.add_scene::<studio::cube::CubeScene>();
        studio_var.add_scene::<studio::bunnymark::BunnyMarkScene>();
        studio_var.add_scene::<studio::texture_example::TextureExample>();
        studio_var.initialize_scene(2);
        self.studio = Some(studio_var);
    }
}

mod game_event_handle;

mod painter;

mod utils;
