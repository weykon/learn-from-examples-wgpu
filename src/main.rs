use egui::EguiRenderer;
use egui_winit::winit::window::Window as WinitWindow;
use std::{
    borrow::BorrowMut,
    cell::RefCell,
    ops::Deref,
    rc::Rc,
    sync::{Arc, Mutex, Weak},
};
use studio::AsAny;
use time_world::FrameCounter;
use winit::{
    dpi::PhysicalSize,
    event_loop::{self, EventLoop},
    window::Window,
};
mod egui;
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
    pub last_update: std::time::Instant,
    pub gui: Option<Arc<Mutex<EguiRenderer>>>,
    pub frame_counter: time_world::FrameCounter,
    pub delta_time: Arc<Mutex<f32>>,
    pub time: Arc<Mutex<f32>>,
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
        studio_var.add_scene::<studio::depth_buffer_example::DepthBufferExample, _>(());
        studio_var.add_scene::<studio::circle_instances::CircleInstancesScene, _>(());
        studio_var
            .add_scene::<studio::uniform_matrix_and_transform_in_shader::UniformMatrixAtGpu, _>(());
        studio_var.add_scene::<studio::instances::InstanceScene, _>(());
        studio_var
            .add_scene::<egui::first::GUISceneExample, (Arc<Window>, Arc<Mutex<EguiRenderer>>,Rc<RefCell<f32>>)>((
                self.window.clone(),
                self.gui.as_ref().unwrap().clone(),
                self.frame_counter.fps.clone(),
            ));
        studio_var.add_scene::<studio::cube::CubeScene, _>(());
        studio_var.add_scene::<studio::bunnymark::BunnyMarkScene, _>(());
        studio_var.add_scene::<studio::texture_example::TextureExample, _>(());
        studio_var.initialize_scene(self.scene_index);
        self.studio = Some(studio_var);
    }
    fn new(window: Arc<Window>, context: Arc<Mutex<gfx::GfxContext>>) -> Self {
        Self {
            window,
            context: context.clone(),
            scene_index: 0,
            studio: None,
            last_update: std::time::Instant::now(),
            gui: None,
            frame_counter: time_world::FrameCounter::new(),
            delta_time: Arc::new(Mutex::new(0.0)),
            time: Arc::new(Mutex::new(0.0)),
        }
    }
    fn mount_next_scene(&mut self) {
        if let Some(studio) = &mut self.studio {
            let next_scene_index = (self.scene_index + 1) % studio.ready_functions.len();
            self.scene_index = next_scene_index;
            studio.initialize_scene(next_scene_index);
        }
    }
    fn set_gui(&mut self) {
        let context = self.context.lock().unwrap();
        let window_binding = self.window.clone();
        let window = window_binding.as_ref();
        let egui = EguiRenderer::new(
            &context.device,
            context.surface_config.as_ref().unwrap().format,
            None,
            1,
            window,
        );
        self.gui = Some(Arc::new(Mutex::new(egui)));
    }
    fn update_game(&mut self, dt: f32, time: f32) {
        println!("update_game");
        {
            let game_ref = self as *const Self;
            self.frame_counter
                .borrow_mut()
                .update(unsafe { &*game_ref });
            println!(
                "update_game : frame_counter updated: {}",
                self.frame_counter.fps.clone().borrow()
            );
            *self.delta_time.lock().unwrap() = dt;
            *self.time.lock().unwrap() = time;
        }
    }
}

mod game_event_handle;

mod painter;

mod utils;
