use std::{borrow::Cow, task::ready};

use bytemuck::Zeroable;
use wgpu::{
    util::DeviceExt, StoreOp, TextureFormat,
};

// some vertex and indexes and instance data
use crate::painter::{Painter, Sandy};
#[repr(C, align(256))]
#[derive(Clone, Copy, Zeroable)]
struct Instance {
    position: [f32; 3],
    color: [f32; 3],
    speed: f32,
    direction: [f32; 2],
}
fn gen_static_data() -> (Vec<f32>, Vec<u16>) {
    #[rustfmt::skip]
    #[allow(non_snake_case)]
    let VERTEX_DATA = [
        -0.5, -0.5, 0.0,   0.2,0.1,0.4,   0.3,  0.1,0.1,
        -0.5, 0.5, 0.0,   0.3,0.1,0.4,   0.1,  0.1,0.1,
        0.5, 0.5, 0.0,   0.2,0.1,0.2,   0.2,  0.1,0.1,
        0.5, -0.5, 0.0,   0.4,0.5,0.1,   0.4,  0.1,0.1,
    ];
    #[rustfmt::skip]
    #[allow(non_snake_case)]
    let INDEX_DATA: [u16;6] = [
        0, 1, 2,   0, 3, 2
    ];
    (VERTEX_DATA.to_vec(), INDEX_DATA.to_vec())
}
impl Sandy for InstanceScene {
    type Extra = ();
    fn ready(
        context: &crate::gfx::GfxContext,
        _: Self::Extra,
    ) -> Self {
        #[allow(non_snake_case)]
        let (VERTEX_DATA, INDEX_DATA) =
            gen_static_data();
        let vertex_buffer = context
            .device
            .create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(
                    &VERTEX_DATA,
                ),
                usage: wgpu::BufferUsages::VERTEX,
            },
        );
        let index_buffer = context
            .device
            .create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(
                    &INDEX_DATA,
                ),
                usage: wgpu::BufferUsages::INDEX,
            },
        );
        let InstancePipeline {
            shader_module,
            pipeline,
            vertex_buffer,
            index_buffer,
        } = InstancePipeline::ready(
            context,
            (vertex_buffer, index_buffer),
        );

        Self {
            pipeline: InstancePipeline {
                shader_module,
                pipeline,
                vertex_buffer,
                index_buffer,
            },
        }
    }
}

struct InstancePipeline {
    shader_module: wgpu::ShaderModule,
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Sandy for InstancePipeline {
    type Extra = (wgpu::Buffer, wgpu::Buffer);
    fn ready(
        context: &crate::gfx::GfxContext,
        extra: Self::Extra,
    ) -> Self
    where
        Self: Sized,
    {
        let (vertex_buffer, index_buffer) = extra;
        let shader_module =
            context.device.create_shader_module(
                wgpu::ShaderModuleDescriptor {
                    label: Some("Shader"),
                    source:
                        wgpu::ShaderSource::Wgsl(
                            Cow::Borrowed(
                                include_str!(
                        "shader/instance.wgsl"
                    ),
                            ),
                        ),
                },
            );
        let pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                // 这里处理顶点缓冲区的布局，而非顶点源数据
                // 而是在render_pass中再写入顶点数据
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 9 * 4,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 3 * 4,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32,
                            offset: 6 * 4,
                            shader_location: 2,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset:  7 * 4,
                            shader_location: 3,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[
                    Some(wgpu::ColorTargetState {
                    format: context.surface_config.as_ref().unwrap().format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache:None,
        });

        Self {
            shader_module,
            pipeline,
            vertex_buffer,
            index_buffer,
        }
    }
}

pub struct InstanceScene {
    pipeline: InstancePipeline,
}
impl Painter for InstanceScene {
    fn paint(
        &mut self,
        context: &crate::gfx::GfxContext,
    ) {
        let frame = context
            .surface
            .get_current_texture()
            .unwrap();
        let view = frame.texture.create_view(
            &wgpu::TextureViewDescriptor::default(
            ),
        );
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });
            println!(
                "InstanceScene::paint: {:?}",
                self.pipeline.vertex_buffer
            );
            println!(
                "InstanceScene::paint: {:?}",
                self.pipeline.index_buffer
            );
            rpass.set_pipeline(
                &self.pipeline.pipeline,
            );

            // 这里的set是传入经过buffer处理源数据后的数据
            rpass.set_vertex_buffer(
                0,
                self.pipeline
                    .vertex_buffer
                    .slice(..),
            );
            rpass.set_index_buffer(
                self.pipeline
                    .index_buffer
                    .slice(..),
                wgpu::IndexFormat::Uint16,
            );
            rpass.draw_indexed(0..6, 0, 0..1);
        }

        context
            .queue
            .submit(Some(encoder.finish()));
        frame.present();
    }
}
