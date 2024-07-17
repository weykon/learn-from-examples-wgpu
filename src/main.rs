use bytemuck::{Pod, Zeroable};
use model::{create_texels, create_vertices, generate_matrix};
use std::{borrow::Cow, sync::Arc};
use wgpu::{
    util::DeviceExt, Buffer, PipelineLayout, RequestAdapterOptions, RequestAdapterOptionsBase,
    Surface, SurfaceTexture, Texture, TextureView, VertexBufferLayout,
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::Event,
    event_loop::{self, EventLoop},
    window::{Window, WindowAttributes},
};
mod model;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut game = GameEntry::Loading;
    let _ = event_loop.run_app(&mut game);
}

enum GameEntry {
    Ready(Game),
    Loading,
}
struct Game {
    window: Arc<Window>,
    context: GfxContext,
    painter: Option<Painter>,
}

struct GfxContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    surface_config: Option<wgpu::SurfaceConfiguration>,
}
impl GfxContext {
    async fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::default();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let surface = unsafe { instance.create_surface(window.clone()).unwrap() };

        GfxContext {
            device,
            queue,
            surface,
            adapter,
            surface_config: None,
        }
    }
}
impl Game {
    fn resumed(&self) {}
    fn bridge_with_gfx(&mut self, PhysicalSize::<u32> { width, height }: PhysicalSize<u32>) {
        let mut surface_config = self
            .context
            .surface
            .get_default_config(&self.context.adapter, width, height)
            .unwrap();
        self.context
            .surface
            .configure(&self.context.device, &surface_config);
        let view_format = surface_config.format.add_srgb_suffix();
        surface_config.view_formats.push(view_format);
        self.context.surface_config = Some(surface_config);
    }
    fn call_painter(&mut self) {
        let painter = Painter::ready_source(&self.context);
        self.painter = Some(painter);
    }
}

impl ApplicationHandler for GameEntry {
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        println!("Resumed");
        match self {
            GameEntry::Ready(game) => {
                // game.resumed(event_loop);
            }
            GameEntry::Loading => {
                let window = Arc::new(
                    event_loop
                        .create_window(WindowAttributes::default())
                        .unwrap(),
                );
                pollster::block_on(async move {
                    println!("in async : Loading");
                    let context = GfxContext::new(window.clone()).await;
                    let game = Game {
                        window,
                        context,
                        painter: None,
                    };
                    *self = GameEntry::Ready(game);
                    println!("in async : Ready");
                });
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let GameEntry::Ready(game) = self {
            match event {
                winit::event::WindowEvent::Resized(size) => {
                    println!("Resized");
                    game.bridge_with_gfx(size);
                    // now arrivate the normal full size in window
                    game.call_painter();
                    game.window.request_redraw();
                }
                winit::event::WindowEvent::Moved(_) => {
                    println!("Moved")
                }
                winit::event::WindowEvent::CloseRequested => {
                    println!("CloseRequested")
                }
                winit::event::WindowEvent::Destroyed => {
                    println!("Destroyed")
                }
                winit::event::WindowEvent::Focused(_) => {
                    println!("Focused")
                }
                winit::event::WindowEvent::KeyboardInput {
                    device_id,
                    event,
                    is_synthetic,
                } => {}
                winit::event::WindowEvent::RedrawRequested => {
                    println!("RedrawRequested");
                    let painter = game.painter.as_ref().unwrap();
                    painter.paint(&game.context);
                }
                _ => {}
            }
        }
    }
}

struct Painter {
    vertex_source: VertexBuff,
    texture_source: TextureBuff,
    uniform_buf: Buffer,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    pipeline_layout: PipelineLayout,
}

impl Painter {
    /// ready sandy thing and blueprints
    fn ready_source(context: &GfxContext) -> Self {
        // vertex_buf, index_buf, vertex_size
        let vertex_source = VertexBuff::ready(context);
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
        let texture_source = TextureBuff::ready(context);

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
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
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
            });

        Painter {
            vertex_source,
            texture_source,
            uniform_buf,
            bind_group,
            pipeline,
            pipeline_layout,
        }
    }
    fn paint(&self, context: &GfxContext) {
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

trait Sandy {
    fn ready(context: &GfxContext) -> Self;
}
struct VertexBuff {
    vertex_buf: Buffer,
    index_buf: Buffer,
    vertex_size: usize,
    index_count: usize,
}
impl Sandy for VertexBuff {
    fn ready(context: &GfxContext) -> Self {
        use wgpu::util::DeviceExt;
        // ready vertex buffer
        let vertex_size: usize = std::mem::size_of::<Vertex>();
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
struct TextureBuff {
    texture: Texture,
    texels: Vec<u8>,
    texture_extent: wgpu::Extent3d,
    texture_view: TextureView,
    size: u32,
}
impl Sandy for TextureBuff {
    fn ready(context: &GfxContext) -> Self {
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

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 2],
}
fn vertex(pos: [i8; 3], tc: [i8; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        _tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}
