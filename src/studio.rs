use std::{
    cell::RefCell,
    ops::Deref,
    rc::Rc,
    sync::{Arc, Mutex, Weak},
};

// 画室
// Gallery
use crate::{
    gfx::{self},
    painter::{Painter, Sandy},
    Game,
};
pub mod bunnymark;
pub mod cube;
pub mod instances;
pub mod texture_example;
pub mod uniform_matrix_and_transform_in_shader;
pub mod circle_instances;

pub struct Studio {
    context: Arc<Mutex<gfx::GfxContext>>,
    pub(crate) ready_functions: Vec<Box<dyn Fn(&gfx::GfxContext) -> Box<dyn Painter>>>,
    current_scene: Option<Rc<RefCell<Box<dyn Painter>>>>,
}
impl Studio {
    pub(crate) fn new(context: Arc<Mutex<gfx::GfxContext>>) -> Self {
        Studio {
            context,
            ready_functions: Vec::new(),
            current_scene: None,
        }
    }

    pub(crate) fn add_scene<T, E>(&mut self, extra: E)
    where
        T: Sandy<Extra = E> + Painter + 'static,
        E: 'static + Clone,
    {
        // 处理 Copy 类型
        if std::mem::needs_drop::<E>() {
            // 非 Copy 类型
            let cloneable_extra = Rc::new(extra);
            self.ready_functions.push(Box::new(move |context| {
                let scene = <T as Sandy>::ready(context, (*cloneable_extra).clone());
                Box::new(scene) as Box<dyn Painter>
            }));
        } else {
            // Copy 类型
            self.ready_functions.push(Box::new(move |context| {
                let scene = <T as Sandy>::ready(context, extra.clone());
                Box::new(scene) as Box<dyn Painter>
            }));
        }
    }

    pub fn initialize_scene(&mut self, index: usize) {
        if let Some(ready_fn) = self.ready_functions.get(index % self.ready_functions.len()) {
            let context_ref = &self.context.as_ref().lock().unwrap();
            let scene = ready_fn(context_ref);
            self.current_scene = Some(Rc::new(RefCell::new(scene)));
        }
    }

    pub fn render_current_scene(&self, dt: f32, time: f32) {
        if let Some(scene) = &self.current_scene {
            let context = self.context.lock().unwrap();
            scene.borrow_mut().paint(&context, dt, time);
        }
    }
}

use core::any::Any;
pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
struct Wrapper<T>(T);

// 为引用类型实现 From trait
impl<T: Clone> From<&T> for Wrapper<T> {
    fn from(t: &T) -> Self {
        Wrapper(t.clone())
    }
}
