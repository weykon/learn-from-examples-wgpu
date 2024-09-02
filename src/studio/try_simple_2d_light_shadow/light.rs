use wgpu::util::DeviceExt;

pub struct Light {
    pub light_buffer: wgpu::Buffer,
    pub light_bind_group: wgpu::BindGroup,
    pub light_bind_group_layout: wgpu::BindGroupLayout,
}

impl Light {
    pub fn new(context: &crate::gfx::GfxContext) -> Self {
        // as uniform
        // light position
        // light color
        // light intensity

        let buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("lgiht buffer"),
                contents: bytemuck::cast_slice(&[-0.1f32,-0.1f32]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            min_binding_size: wgpu::BufferSize::new(8),
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                        },
                        count: None,
                    }],
                    label: Some("light bind group layout"),
                });
        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
                label: Some("light bind group"),
            });

        Light {
            light_buffer: buffer,
            light_bind_group: bind_group,
            light_bind_group_layout: bind_group_layout,
        }
    }
}
