use crate::{
    gfx,
    painter::{Painter as ScenePainter, Sandy},
    time_world::FrameCounter,
};
use egui::{Align2, Context};
use std::{
    borrow::Borrow,
    cell::RefCell,
    iter,
    ops::Deref,
    rc::Rc,
    sync::{Arc, Mutex},
};
use winit::window::Window;

use super::EguiRenderer;

pub fn first(
    ui: &Context,
    fps: Rc<RefCell<f32>>,
) {
    println!("FPS: {}", fps.as_ref().borrow());
    egui::Window::new("Streamline CFD")
        // .vscroll(true)
        .default_open(true)
        .max_width(1000.0)
        .max_height(800.0)
        .default_width(800.0)
        .resizable(true)
        .anchor(Align2::LEFT_TOP, [0.0, 0.0])
        .show(&ui, |mut ui| {
            if ui.add(egui::Button::new("Click me")).clicked() {
                println!("PRESSED")
            }

            ui.label(format!("FPS: {:.2}", fps.as_ref().borrow().deref()));
            // ui.add(egui::Slider::new(_, 0..=120).text("age"));
            ui.end_row();

            // proto_scene.egui(ui);
        });
}

pub struct GUISceneExample {
    pub window: Arc<Window>,
    pub egui: Arc<Mutex<EguiRenderer>>,
    pub fps: Rc<RefCell<f32>>,
}
impl Sandy for GUISceneExample {
    type Extra = (
        Arc<Window>,
        Arc<Mutex<EguiRenderer>>,
        Rc<RefCell<f32>>,
    );
    fn ready(
        context: &gfx::GfxContext,
        (window, egui, fps): Self::Extra,
    ) -> Self
    where
        Self: Sized,
    {
        Self { window, egui, fps }
    }
}

impl ScenePainter for GUISceneExample {
    fn paint(
        &mut self,
        context: &gfx::GfxContext,
    ) {
        let mut encoder = context
            .device
            .create_command_encoder(
                &wgpu::CommandEncoderDescriptor {
                    label: Some("egui encoder"),
                },
            );
        let frame = context
            .surface
            .get_current_texture()
            .unwrap();
        let view = frame.texture.create_view(
            &wgpu::TextureViewDescriptor::default(
            ),
        );
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    view: &view,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        let config = context
            .surface_config
            .as_ref()
            .unwrap();
        let screen_descriptor =
            egui_wgpu::ScreenDescriptor {
                size_in_pixels: [
                    config.width,
                    config.height,
                ],
                pixels_per_point: self
                    .window
                    .scale_factor()
                    as f32,
            };

        self.egui.clone().lock().unwrap().draw(
            &context.device,
            &context.queue,
            &mut encoder,
            &self.window,
            &view,
            screen_descriptor,
            |ui| first(ui, self.fps.clone()),
        );

        context
            .queue
            .submit(iter::once(encoder.finish()));
        frame.present();
    }
}
