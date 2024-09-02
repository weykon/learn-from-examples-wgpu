use wgpu::util::DeviceExt;

pub struct Mesh {
    pub vbuffer: wgpu::Buffer,
    pub ibuffer: wgpu::Buffer,
}
impl Mesh {
    pub fn new(context: &crate::gfx::GfxContext) -> Self {
        let v = gen_v();
        let i: [u16; 6] = [0u16, 2, 1, 1, 2, 3];
        let vbuffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("square"),
                contents: bytemuck::cast_slice(&v),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let ibuffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("square index"),
                contents: bytemuck::cast_slice(&i),
                usage: wgpu::BufferUsages::INDEX,
            });
        Mesh { vbuffer, ibuffer }
    }
}

fn gen_v() -> [f32; 8] {
    [0., 0., 0.0, 0.5, 0.5, 0.0, 0.5, 0.5]
}
