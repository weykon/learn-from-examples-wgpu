use bytemuck::{Pod, Zeroable};
use wgpu::{core::device::queue, util::DeviceExt, FragmentState, VertexState};

use crate::{
    gfx,
    painter::{Painter, Sandy},
};
mod sources;
pub struct CircleInstancesScene {
    vertex_buffer: wgpu::Buffer,
    indexes_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    vertexes_data_size: usize,
    instance_count: usize,
}

const CIRCLE_SEGMENTS: u32 = 360 / 2;
fn gen_vertexes() -> (Vec<f32>, Vec<u32>) {
    let mut vs: Vec<f32> = Vec::new();
    let mut indexes = Vec::new();
    for i in 0..=CIRCLE_SEGMENTS {
        vs.extend_from_slice(&[0., 0.]);
        let angle = i as f32 / CIRCLE_SEGMENTS as f32 * 2.0 * std::f32::consts::PI;
        let x = angle.cos();
        let y = angle.sin();
        vs.push(x);
        vs.push(y);
        let angle = (i + 1) as f32 / CIRCLE_SEGMENTS as f32 * 2.0 * std::f32::consts::PI;
        let x = angle.cos();
        let y = angle.sin();
        vs.push(x);
        vs.push(y);
    }
    for i in 0..(3 * CIRCLE_SEGMENTS) {
        indexes.push(i);
    }
    (vs, indexes)
}

fn gen_instance() -> Vec<Instance> {
    vec![
        Instance {
            radius: 0.1,
            position: [-0.4, 0.],
        },
        Instance {
            radius: 0.08,
            position: [-0.1, 0.],
        },
        Instance {
            radius: 0.05,
            position: [0.2, 0.],
        },
        Instance {
            radius: 0.02,
            position: [0.4, 0.],
        },
    ]
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Instance {
    radius: f32,
    position: [f32; 2],
}

impl Sandy for CircleInstancesScene {
    type Extra = ();
    fn ready(context: &gfx::GfxContext, _extra: Self::Extra) -> Self {
        let (vertex_data, indexes_data) = gen_vertexes();
        let vertexes_data_size = vertex_data.len() * std::mem::size_of::<f32>();
        let instances_data = gen_instance();
        let instance_count = instances_data.len();
        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let indexes_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Indexes Buffer"),
                contents: bytemuck::cast_slice(&indexes_data),
                usage: wgpu::BufferUsages::INDEX,
            });
        let instance_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instances_data.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader Module"),
                source: wgpu::ShaderSource::Wgsl(include_str!("circle_instances.wgsl").into()),
            });
        let (uniform_buffer, bind_group, uniform_bind_group_layout) = TimeUniforms::ready(context);
        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Pipeline Layout"),
                    bind_group_layouts: &[&uniform_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            // 每个顶点的大小
                            array_stride: 2 * 4 as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 0,
                            }],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: 3 * 4 as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32,
                                    offset: 0,
                                    shader_location: 1,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x2,
                                    offset: 4,
                                    shader_location: 2,
                                },
                            ],
                        },
                    ],
                    compilation_options: Default::default(),
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: context.surface_config.as_ref().unwrap().format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });
        CircleInstancesScene {
            vertex_buffer,
            indexes_buffer,
            instance_buffer,
            vertexes_data_size,
            instance_count,
            pipeline,
            bind_group,
            uniform_buffer,
        }
    }
}

impl Painter for CircleInstancesScene {
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

        context.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[TimeUniforms {
                delta_time: dt,
                time,
            }]),
        );
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.indexes_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(
                0..CIRCLE_SEGMENTS * 3 as u32,
                0,
                0..self.instance_count as u32,
            );
        }
        context.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct TimeUniforms {
    delta_time: f32,
    time: f32,
}
impl TimeUniforms {
    fn ready(
        context: &crate::gfx::GfxContext,
    ) -> (wgpu::Buffer, wgpu::BindGroup, wgpu::BindGroupLayout) {
        let uniform_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("uniform_buffer"),
                contents: bytemuck::cast_slice(&[TimeUniforms {
                    delta_time: 0.0,
                    time: 0.0,
                }]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let uniform_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("uniform_bind_group_layout"),
                });
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
                label: Some("uniform_bind_group"),
            });

        (uniform_buffer, bind_group, uniform_bind_group_layout)
    }
}
