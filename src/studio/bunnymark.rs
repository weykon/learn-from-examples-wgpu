use std::{borrow::Cow, ops::Deref, rc::Rc};

use bytemuck::{Pod, Zeroable};
use nanorand::{Rng, WyRand};
use wgpu::{util::DeviceExt, FragmentState};

use crate::painter::{Painter, Sandy};

const MAX_BUNNIES: usize = 1 << 20;
const BUNNY_SIZE: f32 = 0.15 * 256.0;
const GRAVITY: f32 = -9.8 * 100.0;
const MAX_VELOCITY: f32 = 750.0;

impl Sandy for BunnyMarkScene {
    type Extra = ();
    fn ready(context: &crate::gfx::GfxContext, _: Self::Extra) -> Self {
        // 开始前，理清楚一下有什么内容
        // global的一些变量和buffer资源
        // texture的buffer写入和布局
        // 生成一下顶点情况
        let config = context.surface_config.as_ref().unwrap();
        let GlobalThing {
            global_bind_group_layout,
            uniform_alignment,
            config,
            global_group,
        } = GlobalThing::ready(context, config);
        let LocalThing {
            local_bind_group_layout,
            local_buffer,
            local_group,
        } = LocalThing::ready(context, uniform_alignment);
        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("pipeline_layout"),
                    bind_group_layouts: &[&global_bind_group_layout, &local_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                    "shader/bunnymark.wgsl"
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
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.view_formats[0],
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::default(),
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: Some(wgpu::IndexFormat::Uint16),
                    ..wgpu::PrimitiveState::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let rng = WyRand::new_seed(42);

        let mut ins = BunnyMarkScene {
            pipeline,
            global_group,
            local_group,
            bunnies: Vec::new(),
            local_buffer,
            extent: [config.width, config.height],
            rng,
        };

        let spawn_count = 64;
        let color = ins.rng.generate::<u32>();
        println!(
            "Spawning {} bunnies, total at {}",
            spawn_count,
            ins.bunnies.len() + spawn_count
        );
        for _ in 0..spawn_count {
            let speed = ins.rng.generate::<f32>() * MAX_VELOCITY - (MAX_VELOCITY * 0.5);
            ins.bunnies.push(Bunny {
                position: [0.0, 0.5 * (ins.extent[1] as f32)],
                velocity: [speed, 0.0],
                color,
                _pad: 0,
            });
        }

        ins
    }
}

pub struct BunnyMarkScene {
    global_group: wgpu::BindGroup,
    local_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    bunnies: Vec<Bunny>,
    local_buffer: wgpu::Buffer,
    extent: [u32; 2],
    rng: WyRand,
}
impl Painter for BunnyMarkScene {
    fn paint(&mut self, context: &crate::gfx::GfxContext) {
        let delta = 0.01;
        for bunny in self.bunnies.iter_mut() {
            bunny.update_data(delta, &self.extent);
        }

        let uniform_alignment = context.device.limits().min_uniform_buffer_offset_alignment;
        context.queue.write_buffer(&self.local_buffer, 0, unsafe {
            std::slice::from_raw_parts(
                self.bunnies.as_ptr() as *const u8,
                self.bunnies.len() * uniform_alignment as usize,
            )
        });

        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let frame = context.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        {
            let clear_color = wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            };
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.global_group, &[]);
            for i in 0..self.bunnies.len() {
                let offset =
                    (i as wgpu::DynamicOffset) * (uniform_alignment as wgpu::DynamicOffset);
                rpass.set_bind_group(1, &self.local_group, &[offset]);
                rpass.draw(0..4, 0..1);
            }
        }
        context.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    mvp: [[f32; 4]; 4],
    size: [f32; 2],
    pad: [f32; 2],
}
#[repr(C, align(256))]
#[derive(Clone, Copy, Zeroable)]
struct Bunny {
    position: [f32; 2],
    velocity: [f32; 2],
    color: u32,
    _pad: u32,
}
impl Bunny {
    fn update_data(&mut self, delta: f32, extent: &[u32; 2]) {
        self.position[0] += self.velocity[0] * delta;
        self.position[1] += self.velocity[1] * delta;
        self.velocity[1] += GRAVITY * delta;
        if (self.velocity[0] > 0.0 && self.position[0] + 0.5 * BUNNY_SIZE > extent[0] as f32)
            || (self.velocity[0] < 0.0 && self.position[0] - 0.5 * BUNNY_SIZE < 0.0)
        {
            self.velocity[0] *= -1.0;
        }
        if self.velocity[1] < 0.0 && self.position[1] < 0.5 * BUNNY_SIZE {
            self.velocity[1] *= -1.0;
        }
    }
}
/// 主要
struct GlobalThing<'a> {
    global_bind_group_layout: wgpu::BindGroupLayout,
    uniform_alignment: wgpu::BufferAddress,
    config: &'a wgpu::SurfaceConfiguration,
    global_group: wgpu::BindGroup,
}
impl<'a> Sandy for GlobalThing<'a> {
    type Extra = &'a wgpu::SurfaceConfiguration;
    fn ready(context: &crate::gfx::GfxContext, config: Self::Extra) -> Self
    where
        Self: Sized,
    {
        // 世界矩阵的初始数据配置
        let globals = Globals {
            mvp: glam::Mat4::orthographic_rh(
                0.0,
                config.width as f32,
                0.0,
                config.height as f32,
                -1.0,
                1.0,
            )
            .to_cols_array_2d(),
            size: [BUNNY_SIZE; 2],
            pad: [0.0; 2],
        };

        let global_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("global"),
                contents: bytemuck::bytes_of(&globals),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            });

        let global_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("global_bind_group_layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(
                                    std::mem::size_of::<Globals>() as u64,
                                ),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });
        let texture = {
            let img_data = include_bytes!("../icon512.png");
            let decoder = png::Decoder::new(std::io::Cursor::new(img_data));
            let mut reader = decoder.read_info().unwrap();
            let mut buf = vec![0; reader.output_buffer_size()];
            let info = reader.next_frame(&mut buf).unwrap();

            let size = wgpu::Extent3d {
                width: info.width,
                height: info.height,
                depth_or_array_layers: 1,
            };
            let texture = context.device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            context.queue.write_texture(
                texture.as_image_copy(),
                &buf,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(info.width * 4),
                    rows_per_image: None,
                },
                size,
            );
            texture
        };

        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        // 全局到shader的bind_group
        let global_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &global_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: global_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: None,
            });
        // 世界buffer的声明，这里使用的是create_buffer_init
        // init是包括了多一个初始化的参数步骤
        // 可以减少后续数据的上传步骤
        // 世界uniform到shader
        let uniform_alignment =
            context.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;

        Self {
            global_bind_group_layout,
            uniform_alignment,
            config,
            global_group,
        }
    }
}

struct LocalThing {
    local_bind_group_layout: wgpu::BindGroupLayout,
    local_buffer: wgpu::Buffer,
    local_group: wgpu::BindGroup,
}

impl Sandy for LocalThing {
    type Extra = (wgpu::BufferAddress);
    fn ready(context: &crate::gfx::GfxContext, (uniform_alignment): Self::Extra) -> Self
    where
        Self: Sized,
    {
        let local_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("local_bind_group_layout"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            // 可以在同一个缓冲区内存储多个对象的数据，而不是为每个对象创建单独的缓冲区。这样可以减少内存的占用和提高内存的使用效率。
                            has_dynamic_offset: true,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        // 这里使用的是create_buffer
        // 比init少这个初始化的步骤，所以后续需要数据的传入去上传
        let local_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("local"),
            size: (MAX_BUNNIES as wgpu::BufferAddress) * uniform_alignment,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        // 是对齐世界矩阵的group
        let local_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &local_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &local_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<Bunny>() as _),
                    }),
                }],
                label: None,
            });
        Self {
            local_bind_group_layout,
            local_buffer,
            local_group,
        }
    }
}
