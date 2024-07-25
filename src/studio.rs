use std::{
    any::Any,
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

// 画室
// Gallery
use crate::{
    gfx::{self},
    painter::{Painter, Sandy},
};
pub mod bunnymark;
pub mod cube;
pub mod instances;
pub mod texture_example;

pub struct Studio {
    context: Arc<Mutex<gfx::GfxContext>>,
    ready_functions: Vec<Box<dyn Fn(&gfx::GfxContext) -> Box<dyn Painter>>>,
    current_scene: Option<Rc<RefCell<Box<dyn Painter>>>>,
}
impl Studio {
    pub fn new(context: Arc<Mutex<gfx::GfxContext>>) -> Self {
        Studio {
            context,
            ready_functions: Vec::new(),
            current_scene: None,
        }
    }

    pub fn add_scene<T>(&mut self)
    where
        T: Sandy<Extra = ()> + Painter + 'static,
    {
        self.ready_functions.push(Box::new(|context| {
            let scene = T::ready(context, ());
            Box::new(scene) as Box<dyn Painter>
        }));
    }

    pub fn initialize_scene(&mut self, index: usize) {
        if let Some(ready_fn) = self.ready_functions.get(index) {
            let context_ref = &self.context.as_ref().lock().unwrap();
            let scene = ready_fn(context_ref);
            self.current_scene = Some(Rc::new(RefCell::new(scene)));
        }
    }

    pub fn render_current_scene(&self) {
        if let Some(scene) = &self.current_scene {
            let context = self.context.lock().unwrap();
            scene.borrow_mut().paint(&context);
        }
    }
}
