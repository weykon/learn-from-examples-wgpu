use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3, Vec4Swizzles};
use std::{
    any::Any,
    array,
    borrow::{Borrow, Cow},
    cell::RefCell,
    rc::Rc,
};
use wgpu::{util::DeviceExt, MultisampleState, PipelineCompilationOptions, PipelineLayout};

/// here I wanna basicly scene of shader playground and contain some basic element
use crate::{
    painter::{Painter, Sandy},
    utils::models::gen_plane,
};
pub struct ShaderPlaygroundScene {
    pipeline: wgpu::RenderPipeline,
    vertexes_buffer: wgpu::Buffer,
    indexes_buffer: wgpu::Buffer,
    env_matrix_uniform_buffer: wgpu::Buffer,
    time_uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    uniform_pipeline_layout: wgpu::PipelineLayout,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    env_matrix: [EnvUniforms; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct EnvUniforms {
    world: [f32; 16],
    view: [f32; 16],
    proj: [f32; 16],
    _padding: [u8; 64], // 添加填充以达到 256 字节
}
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct TimeUniforms {
    time: f32,
}
fn gen_env_world() -> EnvUniforms {
    // 定义摄像机的位置和目标位置
    let eye = Vec3::new(5.0, 5.0, 5.0); // 摄像机位置
    let target: Vec3 = Vec3::new(0.0, 0.0, 0.0); // 目标位置
    let up = Vec3::new(0.0, 1.0, 0.0); // 上方向

    // 计算 LookAt 矩阵
    let view_matrix: Mat4 = Mat4::look_at_rh(eye, target, up);

    // 定义透视投影矩阵
    let proj_matrix: Mat4 = Mat4::perspective_rh(75.0, 1.0, 0.1, 100.0);

    // 将矩阵转换为数组形式
    let view = view_matrix.to_cols_array();
    let proj = proj_matrix.to_cols_array();

    EnvUniforms {
        world: [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
        view,
        proj,
        _padding: [0; 64],
    }
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

        let UniformThing {
            env_matrix_uniform_buffer,
            time_uniform_buffer,
            bind_group,
            pipeline_layout,
            bind_group_layout,
            env_matrix,
        } = UniformThing::ready(context, ());

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
                layout: Some(&pipeline_layout),
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
            env_matrix_uniform_buffer,
            time_uniform_buffer,
            uniform_bind_group: bind_group,
            uniform_pipeline_layout: pipeline_layout,
            uniform_bind_group_layout: bind_group_layout,
            env_matrix,
        }
    }
}

trait Live: Any {
    fn update(&mut self, dt: f32, rate: f32, i: i32);
}
impl Live for ShaderPlaygroundScene {
    fn update(&mut self, dt: f32, rate: f32, i: i32) {
        let view_mat = Mat4::from_cols_array(&self.env_matrix[i as usize].view);
        // 从视图矩阵中提取摄像机位置
        let position = view_mat.inverse().col(3).xyz();
        // 计算当前位置到中心点的向量
        let mut offset = position - Vec3::new(0.0, 0.0, 0.0);
        // 创建绕Y轴的旋转矩阵
        let rotation = Mat4::from_rotation_y(dt * rate);
        // 应用旋转到偏移向量
        offset = rotation.transform_vector3(offset);
        // 计算新的摄像机位置
        let new_position = Vec3::new(0.0, 0.0, 0.0) + offset;
        // 更新视图矩阵
        let new_view = Mat4::look_at_rh(new_position, Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
        // 将新的视图矩阵转换回 [f32; 16]
        self.env_matrix[i as usize].view = new_view.to_cols_array();
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
            render_pass.set_pipeline(&self.pipeline);
            for i in 0..4 {
                self.update(dt, (i as f32 + 1.) * 0.5, i);
                let dynamic_offset =
                    (i as usize * std::mem::size_of::<EnvUniforms>()) as wgpu::BufferAddress;

                let (x, y) = match i {
                    0 => (0.0, 0.0),
                    1 => (0.5, 0.0),
                    2 => (0.0, 0.5),
                    3 => (0.5, 0.5),
                    _ => unreachable!(),
                };
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[dynamic_offset as u32]);
                context.queue.write_buffer(
                    &self.env_matrix_uniform_buffer,
                    dynamic_offset,
                    bytemuck::cast_slice(&[self.env_matrix[i as usize]]),
                );
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

struct UniformThing {
    env_matrix_uniform_buffer: wgpu::Buffer,
    time_uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    pipeline_layout: wgpu::PipelineLayout,
    bind_group_layout: wgpu::BindGroupLayout,
    env_matrix: [EnvUniforms; 4],
}
impl Sandy for UniformThing {
    type Extra = ();

    fn ready(context: &crate::gfx::GfxContext, extra: Self::Extra) -> Self
    where
        Self: Sized,
    {
        let env_matrix = [
            gen_env_world(),
            gen_env_world(),
            gen_env_world(),
            gen_env_world(),
        ];

        let env_matrix_uniform_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Veenv_matrix_uniform_bufferrtex Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: std::mem::size_of::<EnvUniforms>() as wgpu::BufferAddress * 4,
            mapped_at_creation: false,
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
                                has_dynamic_offset: true,
                                min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                    EnvUniforms,
                                >(
                                )
                                    as u64),
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
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &env_matrix_uniform_buffer,
                            offset: 0,
                            size: wgpu::BufferSize::new(std::mem::size_of::<EnvUniforms>() as u64),
                        }),
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
        Self {
            env_matrix_uniform_buffer,
            time_uniform_buffer,
            bind_group,
            pipeline_layout,
            bind_group_layout: uniform_bind_group_layout,
            env_matrix,
        }
    }
}
