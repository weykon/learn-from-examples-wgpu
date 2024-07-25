use std::borrow::Cow;

use wgpu::util::DeviceExt;

use crate::painter::{Painter, Sandy};

pub  struct TextureExample {
  pub    bind_group: wgpu::BindGroup,
  pub    pipeline: wgpu::RenderPipeline,
  pub    vertex_buffer: wgpu::Buffer,
}
impl Sandy for TextureExample {
    type Extra = ();
    fn ready(context: &crate::gfx::GfxContext, _extra: Self::Extra) -> Self {
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
            let texture: wgpu::Texture = context.device.create_texture(&wgpu::TextureDescriptor {
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

        // 会放到bind group里
        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("my icon sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        #[rustfmt::skip]
        let just_4_vertices : [f32;30] = [
            -0.5, -0.5, 0.0    ,0.0, 1.0, 
            0.5, -0.5, 0.0     ,1.0, 1.0, 
            0.5, 0.5, 0.0      ,1.0, 0.0, 
            0.5, 0.5, 0.0      ,1.0, 0.0, 
            -0.5, -0.5, 0.0    ,0.0, 1.0, 
            -0.5, 0.5, 0.0     ,0.0, 0.0, 
        ];  // f32 32位浮点数 4字节, 5个元素, 20字节

        let vertex_buffer = context.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor { label: Some("my no index vertexs buffer"), contents:  bytemuck::cast_slice(&just_4_vertices), usage: wgpu::BufferUsages::VERTEX, }
        );
        let vertex_buffers_layout = [wgpu::VertexBufferLayout {
            array_stride:   20,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 3*4,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }];

        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some("icon bind group layout"),
                });
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: Some("icon bind group"),
            });
            let shader = context.device.create_shader_module(
                wgpu::ShaderModuleDescriptor {
                    label: Some("icon shader"),
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader/texture_example.wgsl"))),
                }
            );
            let pipeline_layout = context.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("icon pipeline layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
            let pipeline =  context.device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("icon pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState{
                        module: &shader,
                        entry_point: "vs_main",
                        compilation_options: Default::default(),
                        buffers: &vertex_buffers_layout,
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleStrip,
                        strip_index_format: None,
                    ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8UnormSrgb,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::default(),
                        })],
                    }),
                    multiview: None,
                }
            );
        Self {
            bind_group,
            pipeline,
            // pass set的
            vertex_buffer
        }

    }
}
impl Painter for TextureExample {
    fn paint(&mut self, context: &crate::gfx::GfxContext) {
         let frame = context.surface.get_current_texture().unwrap();
         let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
         let mut encoder = context.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("icon encoder") },
         );

         { 
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None , 
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
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.draw(0..6, 0..1);
         };

        context.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

    }
}
