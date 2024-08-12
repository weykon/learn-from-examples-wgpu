use std::borrow::Cow;
use wgpu::{core::pipeline, util::DeviceExt, MultisampleState, PipelineCompilationOptions};
use winit::dpi::PhysicalSize;

/// here I wanna basicly scene of shader playground and contain some basic element
use crate::{
    painter::{Painter, Sandy},
    utils::models::gen_plane,
};
pub struct ShaderPlaygroundScene {
    pipeline: wgpu::RenderPipeline,
    vertexes_buffer: wgpu::Buffer,
    indexes_buffer: wgpu::Buffer,
}

// 由于开发的内容较多，所以分阶段，先一个一个实现，开watch
impl Sandy for ShaderPlaygroundScene {
    type Extra = ();
    fn ready(context: &crate::gfx::GfxContext, extra: Self::Extra) -> Self
    where
        Self: Sized,
    {
        let (vertexes, indexes) = gen_plane();
        let vertexes_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(&vertexes),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let indexes_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&indexes),
                usage: wgpu::BufferUsages::INDEX,
            });

        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "shader_playgroud_scene/base.wgsl"
                ))),
            });
        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: None,
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: 6 * 4 as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
                    }],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: context.surface_config.as_ref().unwrap().format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
                cache: None,
            });
        Self {
            pipeline,
            vertexes_buffer,
            indexes_buffer,
        }
    }
}

impl Painter for ShaderPlaygroundScene {
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

        let config = context.surface_config.as_ref().unwrap();
        let (width, height) = (config.width, config.height);
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                label: None,
                depth_stencil_attachment: None,
                ..Default::default()
            });
            for i in 0..4 {
                let (x, y) = match i {
                    0 => (0.0, 0.0),
                    1 => (0.5, 0.0),
                    2 => (0.0, 0.5),
                    3 => (0.5, 0.5),
                    _ => unreachable!(),
                };
                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_viewport(
                    x * width as f32,
                    y * height as f32,
                    width as f32 / 2.0,
                    height as f32 / 2.0,
                    0.0,
                    1.0,
                );
                render_pass.set_vertex_buffer(0, self.vertexes_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.indexes_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..6, 0, 0..1);
            }
        }

        context.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}

// modeling
// a plane, a cube ,a sphere, a circle
