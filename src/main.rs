use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};
use winit::{
    dpi::PhysicalSize,
    event_loop::{self, EventLoop},
    window::Window,
};
mod model;
mod studio;
mod time_world;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut game = GameEntry::Loading;
    let _ = event_loop.run_app(&mut game);
}

enum GameEntry {
    Loading,
    Ready(Game),
}

pub struct Game {
    pub window: Arc<Window>,
    pub(crate) context: Arc<Mutex<gfx::GfxContext>>,
    pub studio: Option<studio::Studio>,
    pub scene_index: usize,
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
        let context: Arc<Mutex<gfx::GfxContext>> = self.context.clone();
        let mut studio_var = studio::Studio::new(context);

        studio_var.add_scene::<studio::cube::CubeScene>();
        studio_var.add_scene::<studio::bunnymark::BunnyMarkScene>();
        studio_var.add_scene::<studio::texture_example::TextureExample>();
        studio_var.initialize_scene(self.scene_index);
        self.studio = Some(studio_var);
    }
    fn new(window: Arc<Window>, context: Arc<Mutex<gfx::GfxContext>>) -> Self {
        Self {
            window,
            context: context.clone(),
            scene_index: 0,
            studio: None,
        }
    }
    fn mount_next_scene(&mut self) {
        if let Some(studio) = &mut self.studio {
            let next_scene_index = (self.scene_index + 1) % studio.ready_functions.len();
            self.scene_index = next_scene_index;
            studio.initialize_scene(next_scene_index);
        }
    }
}

mod game_event_handle;

mod painter;

mod utils;
