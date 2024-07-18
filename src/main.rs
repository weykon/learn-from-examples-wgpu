use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event_loop::{self, EventLoop},
    window::{Window, WindowAttributes},
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
        studio_var.add_scene::<studio::instances::InstanceScene>();
        studio_var.initialize_scene(0);
        self.studio = Some(studio_var);
    }
}

impl ApplicationHandler for GameEntry {
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        println!("Resumed");
        match self {
            GameEntry::Ready(game) => {
                // game.resumed(event_loop);
            }
            GameEntry::Loading => {
                let window = Arc::new(
                    event_loop
                        .create_window(WindowAttributes::default())
                        .unwrap(),
                );
                pollster::block_on(async move {
                    println!("in async : Loading");
                    let context = gfx::GfxContext::new(window.clone()).await;
                    let context = Arc::new(Mutex::new(context));
                    let game = Game {
                        window,
                        context: context.clone(),
                        studio: None,
                    };
                    *self = GameEntry::Ready(game);
                    println!("in async : Ready");
                });
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let GameEntry::Ready(game) = self {
            match event {
                winit::event::WindowEvent::Resized(size) => {
                    println!("Resized");
                    game.bridge_with_gfx(size);
                    // now arrivate the normal full size in window
                    game.list_painter();
                    game.window.request_redraw();
                }
                winit::event::WindowEvent::Moved(_) => {
                    println!("Moved")
                }
                winit::event::WindowEvent::CloseRequested => {
                    println!("CloseRequested")
                }
                winit::event::WindowEvent::Destroyed => {
                    println!("Destroyed")
                }
                winit::event::WindowEvent::Focused(_) => {
                    println!("Focused")
                }
                winit::event::WindowEvent::KeyboardInput {
                    device_id,
                    event,
                    is_synthetic,
                } => {}
                winit::event::WindowEvent::RedrawRequested => {
                    println!("RedrawRequested");
                    game.studio.as_ref().unwrap().render_current_scene();
                }
                _ => {}
            }
        }
    }
}

mod painter;

mod utils;
