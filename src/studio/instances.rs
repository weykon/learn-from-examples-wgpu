use crate::painter::{Painter, Sandy};
impl Sandy for InstanceScene {
    type Extra = ();
    fn ready(context: &crate::gfx::GfxContext, _: Self::Extra) -> Self {
        InstanceScene
    }
}

pub struct InstanceScene;
impl Painter for InstanceScene {
    fn paint(&mut self, context: &crate::gfx::GfxContext) {}
}
