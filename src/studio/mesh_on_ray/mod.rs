use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use wgpu::{util::DeviceExt, MultisampleState, PipelineCompilationOptions, PrimitiveState};

use crate::painter::{Painter, Sandy};

struct MeshOnRay {
    ray: Ray,
    mesh: (wgpu::Buffer, wgpu::Buffer),
    pipeline: wgpu::RenderPipeline,
    ray_bind_group: wgpu::BindGroup,
    time_uniform_buffer: wgpu::Buffer,
    time_bind_group: wgpu::BindGroup,
    instance_count: usize,
    terrain_instances_buffer: wgpu::Buffer,
}
#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy)]
struct Ray {
    origin: [f32; 3],
    direction: [f32; 3],
    intensity: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
enum Terrain {
    Wall = 0,
    Ground = 1,
    Water = 2,
}
#[repr(C)]
#[derive(Copy, Clone, Zeroable, Pod)]
struct MeshInstance {
    position: [f32; 3],
    rotation: [f32; 3],
    on_type: u32,
    scale: f32,
}
impl From<Terrain> for u32 {
    fn from(terrain: Terrain) -> Self {
        terrain as u32
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct TimeUniforms {
    delta_time: f32,
    time: f32,
}

fn gen() -> ([f32; 36], [u16; 6]) {
    #[rustfmt::skip]
    let vs: [f32; 36] = [
        -0.3, -0.3,0.1,       0.0, 0.0, 1.0,
        0.3, 0.3, 0.1,        0.0, 0.0, 1.0,
        -0.3, 0.3,0.1,        0.0, 0.0, 1.0,

        -0.3, 0.3, -0.1,       0.0, 1.0, 0.0,
        -0.3, -0.3, -0.1,      0.0, 1.0, 0.0,
        0.3, -0.3, -0.1,       0.0, 1.0, 0.0,
    ];
    let indexes: [u16; 6] = [0, 1, 2, 3, 4, 5];
    (vs, indexes)
}
fn gen_instances() -> Vec<MeshInstance> {
    let mut instances = Vec::new();
    instances.push(MeshInstance {
        position: [0.1, 0.1, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: 0.1,
        on_type: Terrain::Wall.into(),
    });

    instances.push(MeshInstance {
        position: [0.1, 0.2, 0.0],
        rotation: [0.0, 0.0, 0.0],
        scale: 0.1,
        on_type: Terrain::Wall.into(),
    });
    instances
}
impl Sandy for MeshOnRay {
    type Extra = ();
    fn ready(context: &crate::gfx::GfxContext, extra: Self::Extra) -> Self
    where
        Self: Sized,
    {
        let ray = Ray {
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
            intensity: 1.0,
        };
        let instances = gen_instances();
        let (vs, indexes) = gen();
        let mesh = (
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("vs"),
                    contents: bytemuck::cast_slice(&vs),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("is"),
                    contents: bytemuck::cast_slice(&indexes),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
        );

        let ray_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vs"),
                contents: bytemuck::cast_slice(&[ray]),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let ray_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("ray_bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        let ray_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("ray_bind_group"),
                layout: &ray_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &ray_buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
            });

        let time_uniforms = TimeUniforms {
            delta_time: 0.0,
            time: 0.0,
        };
        let time_uniform_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[time_uniforms]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
        let time_uniform_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Time Bind Group Layout"),
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
        let time_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Time Bind Group"),
                layout: &time_uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &time_uniform_buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
            });

        let terrain_instances_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Terrain Buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Ray Pipeline Layout"),
                    bind_group_layouts: &[&ray_bind_group_layout, &time_uniform_bind_group_layout],
                    push_constant_ranges: &[],
                });

        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("ray_2d"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("ray_2d.wgsl"))),
            });
        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Ray Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "main",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: 6 * std::mem::size_of::<f32>() as wgpu::BufferAddress,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x3,
                                    offset: 0,
                                    shader_location: 0,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x3,
                                    offset: 3 * 4 as wgpu::BufferAddress,
                                    shader_location: 1,
                                },
                            ],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: 32,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x3,
                                    offset: 0,
                                    shader_location: 2,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x3,
                                    offset: 12,
                                    shader_location: 3,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Uint32,
                                    offset: 24,
                                    shader_location: 4,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32,
                                    offset: 28,
                                    shader_location: 5,
                                },
                            ],
                        },
                    ],
                    compilation_options: PipelineCompilationOptions::default(),
                },
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: context.surface_config.as_ref().unwrap().format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                multiview: None,
                cache: None,
            });
        MeshOnRay {
            ray,
            mesh,
            pipeline,
            ray_bind_group,
            time_uniform_buffer,
            time_bind_group,
            instance_count: 2 as usize,
            terrain_instances_buffer,
        }
    }
}

impl Painter for MeshOnRay {
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
            &self.time_uniform_buffer,
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
            render_pass.set_bind_group(0, &self.ray_bind_group, &[]);
            render_pass.set_bind_group(1, &self.time_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.mesh.0.slice(..));
            render_pass.set_vertex_buffer(1, self.terrain_instances_buffer.slice(..));
            render_pass.set_index_buffer(self.mesh.1.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..6, 0, 0..self.instance_count as u32);
        }
        context.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
