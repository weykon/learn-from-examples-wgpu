// one mesh or instance(later)
// one light
// one shadow
mod light;
mod mesh;
mod shadow;
use std::rc::Rc;

use crate::painter::{Painter, Sandy};
use light::Light;
use mesh::Mesh;
use shadow::Shadow;

pub struct Simple2DLightShadow {
    pub mesh: Mesh,
    pub light: Light,
    pub shadow: Shadow,
}

impl Sandy for Simple2DLightShadow {
    type Extra = ();

    fn ready(context: &crate::gfx::GfxContext, extra: Self::Extra) -> Self
    where
        Self: Sized,
    {
        let mesh = Mesh::new(context);
        let light = Light::new(context);
        let shadow = Shadow::ready(context, &mesh,&light);
        Simple2DLightShadow {
            mesh,
            light,
            shadow,
        }
    }
}

impl Painter for Simple2DLightShadow {
    fn paint(&mut self, context: &crate::gfx::GfxContext, dt: f32, time: f32) {
        let frame = context.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let mut encoder = self.shadow.paint(
            context,
            &self.mesh,
            encoder,
            &view,
            &self.light
        );

        context.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
