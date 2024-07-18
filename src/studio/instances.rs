use crate::painter::{Painter, Sandy};

pub struct InstanceScene;
impl Painter for InstanceScene {
    fn paint(&self, context: &crate::gfx::GfxContext) {
        
    }
}

impl Sandy for InstanceScene {
    fn ready(context: &crate::gfx::GfxContext) -> Self {
        todo!()
    }
}
