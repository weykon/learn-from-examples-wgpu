use std::{borrow::Cow, task::ready};

use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, StoreOp, TextureFormat};

// some vertex and indexes and instance data
use crate::painter::{Painter, Sandy};
#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct Instance {
    position: [f32; 3],
}
fn gen_instance_data() -> Vec<Instance> {
    let mut instances: Vec<Instance> = Vec::new();
    let radius = 0.1;
    let num_instances = 5;

    for i in 0..num_instances {
        let angle = (i as f32) * (2.0 * std::f32::consts::PI / num_instances as f32);
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        instances.push(Instance {
            position: [x, y, 0.0],
        });
    }
    instances
}
fn gen_static_data() -> (Vec<f32>, Vec<u16>) {
    #[rustfmt::skip]
    #[allow(non_snake_case)]
    let VERTEX_DATA = [
        -0.1, -0.1, 0.0,    0.2,0.1,0.4, 
        -0.1, 0.1, 0.0,     0.3,0.1,0.4,  
        0.1, 0.1, 0.0,      0.2,0.1,0.2,   
        0.1, -0.1, 0.0,     0.4,0.5,0.1,  
    ];
    #[rustfmt::skip]
    #[allow(non_snake_case)]
    let INDEX_DATA: [u16;6] = [
       2,1,0,   0,3,2
    ];
    (VERTEX_DATA.to_vec(), INDEX_DATA.to_vec())
}
struct UniformTime {
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct Uniforms {
    delta_time: f32,
    time: f32,
}

impl Sandy for UniformTime {
    type Extra = ();
    fn ready(context: &crate::gfx::GfxContext, _: Self::Extra) -> Self {
        let uniform_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[Uniforms {
                    delta_time: 0.0,
                    time: 0.0,
                }]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let uniform_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Uniform Bind Group Layout"),
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
                });
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Uniform Bind Group"),
                layout: &uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
            });

        Self {
            uniform_buffer,
            uniform_bind_group_layout,
            bind_group,
        }
    }
}
impl Sandy for InstanceScene {
    type Extra = ();
    fn ready(context: &crate::gfx::GfxContext, _: Self::Extra) -> Self {
        #[allow(non_snake_case)]
        let (VERTEX_DATA, INDEX_DATA) = gen_static_data();
        let instance_data = gen_instance_data();
        let instances_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instance_data),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&VERTEX_DATA),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&INDEX_DATA),
                usage: wgpu::BufferUsages::INDEX,
            });
        let UniformTime {
            uniform_buffer,
            bind_group,
            uniform_bind_group_layout,
        } = UniformTime::ready(context, ());
        let InstancePipeline {
            shader_module,
            pipeline,
            vertex_buffer,
            index_buffer,
        } = InstancePipeline::ready(
            context,
            (vertex_buffer, index_buffer, uniform_bind_group_layout),
        );

        Self {
            pipeline: InstancePipeline {
                shader_module,
                pipeline,
                vertex_buffer,
                index_buffer,
            },
            instances_buffer,
            uniform_buffer,
            bind_group,
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
    type Extra = (wgpu::Buffer, wgpu::Buffer, wgpu::BindGroupLayout);
    fn ready(context: &crate::gfx::GfxContext, extra: Self::Extra) -> Self
    where
        Self: Sized,
    {
        let (vertex_buffer, index_buffer, uniform_bind_group_layout) = extra;
        let shader_module = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "shader/instance.wgsl"
                ))),
            });
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
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    // 这里处理顶点缓冲区的布局，而非顶点源数据
                    // 而是在render_pass中再写入顶点数据
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: 6 * 4,
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
                            ],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: 3 * 4,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 2,
                            }],
                        },
                    ],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
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
                cache: None,
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
    instances_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}
impl Painter for InstanceScene {
    fn paint(&mut self, context: &crate::gfx::GfxContext, dt: f32, time: f32) {
        let frame = context.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let new_uniform = [Uniforms {
            delta_time: dt,
            time: time,
        }];
        let input_dt: &[u8] = bytemuck::cast_slice(&new_uniform);
        context
            .queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(input_dt));
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
            rpass.set_pipeline(&self.pipeline.pipeline);

            // 这里的set是传入经过buffer处理源数据后的数据
            rpass.set_vertex_buffer(0, self.pipeline.vertex_buffer.slice(..));
            rpass.set_index_buffer(
                self.pipeline.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_vertex_buffer(1, self.instances_buffer.slice(..));
            rpass.draw_indexed(0..6, 0, 0..5);
        }
        context.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
