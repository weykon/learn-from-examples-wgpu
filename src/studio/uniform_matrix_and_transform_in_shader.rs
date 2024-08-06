use std::borrow::Cow;

use bytemuck::{Pod, Zeroable};
use cgmath::{perspective, Deg, Matrix4, Point3, Vector3};
use wgpu::util::DeviceExt;

use crate::painter::{Painter, Sandy};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct EnvUniforms {
    world: [[f32; 4]; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct TimeUniforms {
    time: f32,
}

// 顶点数据
fn gen_vertexes() -> &'static [f32; 120] {
    #[rustfmt::skip]
    let vertex_data :&[f32; 120] = &[
        -1., -1., 1.,           1., 0.,
        1., -1., 1.,            0., 0.,
        1., 1., 1.,             0., 1.,
        -1., 1., 1.,            1., 1.,
        -1., 1., -1.,           1., 0.,
        1., 1., -1.,            0., 0.,
        1., -1., -1.,           0., 1.,
        -1., -1., -1.,          1., 1.,
        1., -1., -1.,           0., 0.,
        1., 1., -1.,            1., 0.,
        1., 1., 1.,             1., 1.,
        1., -1., 1.,            0., 1.,
        -1., -1., 1.,           1., 0.,
        -1., 1., 1.,            0., 0.,
        -1., 1., -1.,           0., 1.,
        -1., -1., -1.,          1., 1.,
        1., 1., -1.,            1., 0.,
        -1., 1., -1.,           0., 0.,
        -1., 1., 1.,            0., 1.,
        1., 1., 1.,             1., 1.,
        1., -1., 1.,            0., 0.,
        -1., -1., 1.,           1., 0.,
        -1., -1., -1.,          1., 1.,
        1., -1., -1.,           0., 1.,
    ];
    vertex_data
}

fn gen_indexes() -> &'static [u16; 36] {
    #[rustfmt::skip]
    let index_data :&[u16; 36] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];
    index_data
}

fn gen_env_by_glam() {
    // let view = glam::Mat4::look_at_rh(self.pos, glam::Vec3::ZERO, glam::Vec3::Z);
    // let projection = glam::Mat4::perspective_rh(
    //     self.fov * consts::PI / 180.,
    //     1.0,
    //     self.depth.start,
    //     self.depth.end,
    // );
    // let view_proj = projection * view;
    // LightRaw {
    //     proj: view_proj.to_cols_array_2d(),
    //     pos: [self.pos.x, self.pos.y, self.pos.z, 1.0],
    //     color: [
    //         self.color.r as f32,
    //         self.color.g as f32,
    //         self.color.b as f32,
    //         1.0,
    //     ],
    // }
}
fn gen_env_world() -> EnvUniforms {
    // 定义摄像机的位置和目标位置
    let eye = Point3::new(5.0, 5.0, 5.0); // 摄像机位置
    let target = Point3::new(0.0, 0.0, 0.0); // 目标位置
    let up = Vector3::new(0.0, 1.0, 0.0); // 上方向

    // 计算 LookAt 矩阵
    let view_matrix: Matrix4<f32> = Matrix4::look_at_rh(eye, target, up);

    // 定义透视投影矩阵
    let proj_matrix: Matrix4<f32> = perspective(Deg(75.0), 1.0, 0.1, 100.0);

    // 将矩阵转换为数组形式
    let view: [[f32; 4]; 4] = view_matrix.into();
    let proj: [[f32; 4]; 4] = proj_matrix.into();

    EnvUniforms {
        world: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
        view,
        proj,
    }
}
pub struct UniformMatrixAtGpu {
    pub vertex_buffer: wgpu::Buffer,
    pub indexes_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::RenderPipeline,
    pub time_uniform_buffer: wgpu::Buffer,
}

impl Sandy for UniformMatrixAtGpu {
    type Extra = ();

    fn ready(context: &crate::gfx::GfxContext, extra: Self::Extra) -> Self
    where
        Self: Sized,
    {
        let vertex_data = gen_vertexes();
        let index_data = gen_indexes();

        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let indexes_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(index_data),
                usage: wgpu::BufferUsages::INDEX,
            });

        let env_matrix_uniform_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Veenv_matrix_uniform_bufferrtex Buffer"),
                    contents: bytemuck::cast_slice(&[gen_env_world()]),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
        let time_uniform_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("time_uniform_buffer Buffer"),
                    contents: bytemuck::cast_slice(&[TimeUniforms { time: 0.0 }]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let uniform_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                    label: Some("uniform_bind_group_layout"),
                });
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &uniform_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            env_matrix_uniform_buffer.as_entire_buffer_binding(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(
                            time_uniform_buffer.as_entire_buffer_binding(),
                        ),
                    },
                ],
                label: Some("bind_group"),
            });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Pipeline Layout"),
                    bind_group_layouts: &[&uniform_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "shader/uniform_matrix_and_transform_in_shader.wgsl"
                ))),
            });
        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: 20,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 0,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 12,
                                shader_location: 1,
                            },
                        ],
                    }],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: context.surface_config.as_ref().unwrap().format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });
        Self {
            vertex_buffer,
            indexes_buffer,
            bind_group,
            pipeline,
            time_uniform_buffer,
        }
    }
}

impl Painter for UniformMatrixAtGpu {
    fn paint(&mut self, context: &crate::gfx::GfxContext, dt: f32, time: f32) {
        let frame = context.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });
        context.queue.write_buffer(
            &self.time_uniform_buffer,
            0,
            bytemuck::cast_slice(&[TimeUniforms { time }]),
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
            render_pass.set_index_buffer(self.indexes_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..36, 0, 0..1);
        }
        context.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
