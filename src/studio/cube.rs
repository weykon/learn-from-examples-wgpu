use std::borrow::Cow;

use wgpu::{util::DeviceExt, Buffer, PipelineLayout};

use crate::{
    gfx::{self, GfxContext},
    model::{create_texels, create_vertices, generate_matrix},
    painter::{Sandy, TextureBuff, VertexBuff},
    utils::{self, Vertex},
};

use super::Painter;

pub(crate) struct CubeScene {
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) pipeline_layout: PipelineLayout,
    pub(crate) texture_source: TextureBuff,
    pub(crate) uniform_buf: Buffer,
    pub(crate) vertex_source: VertexBuff,
}

impl Sandy for CubeScene {
    type Extra = ();
    fn ready(context: &gfx::GfxContext, _: Self::Extra) -> Self {
        // vertex_buf, index_buf, vertex_size
        let vertex_source = VertexBuff::ready(context, ());
        let vertex_buffers_layout = [wgpu::VertexBufferLayout {
            array_stride: vertex_source.vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        }];

        // texture, texels, size
        let texture_source = TextureBuff::ready(context, ());

        // Create other resources
        let config = context.surface_config.as_ref().unwrap();
        let mx_total = generate_matrix(config.width as f32 / config.height as f32);
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        let uniform_buf = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(mx_ref),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // bind group
        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Main Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: wgpu::BufferSize::new(64),
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Uint,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                    ],
                });

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Main Pipeline Layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&texture_source.texture_view),
                    },
                ],
                label: None,
            });
        // pipeline
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader/cube.wgsl"))),
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
                    buffers: &vertex_buffers_layout,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    compilation_options: Default::default(),
                    targets: &[Some(config.view_formats[0].into())],
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

        CubeScene {
            vertex_source,
            texture_source,
            uniform_buf,
            bind_group,
            pipeline,
            pipeline_layout,
        }
    }
}

impl Painter for CubeScene {
    fn paint(&mut self, context: &gfx::GfxContext,dt:f32, time: f32) {
        let frame = context.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
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
            rpass.push_debug_group("Prepare data for draw.");
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_index_buffer(
                self.vertex_source.index_buf.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            rpass.set_vertex_buffer(0, self.vertex_source.vertex_buf.slice(..));
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            rpass.draw_indexed(0..self.vertex_source.index_count as u32, 0, 0..1);
            // if let Some(ref pipe) = self.pipeline_wire {
            //     rpass.set_pipeline(pipe);
            //     rpass.draw_indexed(0..self.vertex_source.index_count as u32, 0, 0..1);
            // }
        }

        context.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

impl Sandy for VertexBuff {
    type Extra = ();
    fn ready(context: &gfx::GfxContext, _: Self::Extra) -> Self {
        use wgpu::util::DeviceExt;
        // ready vertex buffer
        let vertex_size: usize = std::mem::size_of::<utils::Vertex>();
        let (vertex_data, index_data) = create_vertices();
        let vertex_buf = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buf = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&index_data),
                usage: wgpu::BufferUsages::INDEX,
            });

        VertexBuff {
            vertex_buf,
            index_count: index_data.len(),
            index_buf,
            vertex_size,
        }
    }
}

impl Sandy for TextureBuff {
    type Extra = ();
    fn ready(context: &gfx::GfxContext, _: Self::Extra) -> Self {
        let size = 256u32;
        let texels = create_texels(size as usize);
        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        };
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Main Texture"),
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        context.queue.write_texture(
            texture.as_image_copy(),
            &texels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(size),
                rows_per_image: None,
            },
            texture_extent,
        );
        TextureBuff {
            texture,
            texture_view,
            texels,
            texture_extent,
            size,
        }
    }
}
