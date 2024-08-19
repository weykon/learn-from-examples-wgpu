use std::borrow::BorrowMut;

use bytemuck::Pod;
use wgpu::util::DeviceExt;

use super::{vertex, Vertex};

pub fn gen_plane() -> ([f32; 36], [u16; 6]) {
    #[rustfmt::skip]
    let vs: [f32; 36] = [
        -0.3, -0.3,0.1,       0.0, 0.0, 1.0,
        0.3, 0.3, 0.1,        0.0, 0.0, 1.0,
        -0.3, 0.3,0.1,        0.0, 0.0, 1.0,

        -0.3, 0.3, 0.1,       0.0, 1.0, 0.0,
        -0.3, -0.3, 0.1,      0.0, 1.0, 0.0,
        0.3, -0.3, 0.1,       0.0, 1.0, 0.0,
    ];
    let indexes: [u16; 6] = [0, 1, 2, 3, 4, 5];
    (vs, indexes)
}

fn gen_cube() -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        // top (0, 0, 1)
        vertex([-1, -1, 1], [0, 0]),
        vertex([1, -1, 1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([-1, 1, 1], [0, 1]),
        // bottom (0, 0, -1)
        vertex([-1, 1, -1], [1, 0]),
        vertex([1, 1, -1], [0, 0]),
        vertex([1, -1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // right (1, 0, 0)
        vertex([1, -1, -1], [0, 0]),
        vertex([1, 1, -1], [1, 0]),
        vertex([1, 1, 1], [1, 1]),
        vertex([1, -1, 1], [0, 1]),
        // left (-1, 0, 0)
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, 1, 1], [0, 0]),
        vertex([-1, 1, -1], [0, 1]),
        vertex([-1, -1, -1], [1, 1]),
        // front (0, 1, 0)
        vertex([1, 1, -1], [1, 0]),
        vertex([-1, 1, -1], [0, 0]),
        vertex([-1, 1, 1], [0, 1]),
        vertex([1, 1, 1], [1, 1]),
        // back (0, -1, 0)
        vertex([1, -1, 1], [0, 0]),
        vertex([-1, -1, 1], [1, 0]),
        vertex([-1, -1, -1], [1, 1]),
        vertex([1, -1, -1], [0, 1]),
    ];

    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

pub fn gen_sphere(radius: f32, sectors: u32, stacks: u32) -> (Vec<f32>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let sector_step = 2.0 * std::f32::consts::PI / sectors as f32;
    let stack_step = std::f32::consts::PI / stacks as f32;

    for i in 0..=stacks {
        let stack_angle = std::f32::consts::PI / 2.0 - i as f32 * stack_step;
        let xy = radius * stack_angle.cos();
        let z = radius * stack_angle.sin();

        for j in 0..=sectors {
            let sector_angle = j as f32 * sector_step;

            // 顶点位置
            let x = xy * sector_angle.cos();
            let y = xy * sector_angle.sin();
            vertices.extend_from_slice(&[x, y, z]);

            // 法线向量（归一化的顶点坐标）
            let nx = x / radius;
            let ny = y / radius;
            let nz = z / radius;
            vertices.extend_from_slice(&[nx, ny, nz]);

            // 纹理坐标
            let s = j as f32 / sectors as f32;
            let t = i as f32 / stacks as f32;
            vertices.extend_from_slice(&[s, t]);
        }
    }

    // 生成索引
    for i in 0..stacks {
        let mut k1 = i * (sectors + 1);
        let mut k2 = k1 + sectors + 1;

        for j in 0..sectors {
            if i != 0 {
                indices.extend_from_slice(&[k1 as u16, k2 as u16, (k1 + 1) as u16]);
            }
            if i != (stacks - 1) {
                indices.extend_from_slice(&[(k1 + 1) as u16, k2 as u16, (k2 + 1) as u16]);
            }
            k1 += 1;
            k2 += 1;
        }
    }

    (vertices, indices)
}

pub trait Model {
    type V: Sized + 'static + AsRef<[f32]>;
    type I: Sized + 'static + AsRef<[Self::IndexType]>;
    type IndexType: Sized + Copy + 'static + Pod;

    fn gen() -> (Self::V, Self::I);
    fn to_buffer<'a>(
        v: Self::V,
        i: Self::I,
        context: &crate::gfx::GfxContext,
    ) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertexes_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice(v.as_ref()),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let indexes_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(i.as_ref()),
                usage: wgpu::BufferUsages::INDEX,
            });
        (vertexes_buffer, indexes_buffer)
    }
}

pub struct Plane {
    vertexes: Vec<f32>,
    indexes: Vec<u16>,
}

impl Model for Plane {
    fn gen() -> (Vec<f32>, Vec<u16>) {
        let (vertices, indices) = gen_plane();
        (vertices.to_vec(), indices.to_vec())
    }
    type V = Vec<f32>;
    type I = Vec<u16>;
    type IndexType = u16;
}
pub struct Sphere {
    vertexes: Vec<f32>,
    indexes: Vec<u16>,
}
impl Model for Sphere {
    fn gen() -> (Vec<f32>, Vec<u16>) {
        let (vertices, indices) = gen_sphere(0.5, 64, 32);
        (vertices.to_vec(), indices.to_vec())
    }
    type V = Vec<f32>;
    type I = Vec<u16>;
    type IndexType = u16;
}

pub struct Cube {
    vertexes: Vec<f32>,
    indexes: Vec<u16>,
}

impl Model for Cube {
    fn gen() -> (Vec<f32>, Vec<u16>) {
        let (vertices, indices) = gen_cube();
        (
            vertices
                .iter()
                .flat_map(|v| v.to_f32_array().into_iter())
                .collect::<Vec<f32>>(),
            indices.to_vec(),
        )
    }
    type V = Vec<f32>;
    type I = Vec<u16>;
    type IndexType = u16;
}

#[derive(Copy, Clone)]
pub enum ModelType {
    Plane,
    Sphere,
    Cube,
}

pub struct ModelBuffers {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

impl ModelType {
    pub fn iterator() -> impl Iterator<Item = ModelType> {
        [Self::Plane, Self::Sphere, Self::Cube, Self::Plane]
            .iter()
            .copied()
    }
    pub fn create_all_buffers(context: &crate::gfx::GfxContext) -> Vec<ModelBuffers> {
        Self::iterator()
            .map(|model_type| {
                let (vertices, indices) = match model_type {
                    ModelType::Plane => Plane::gen(),
                    ModelType::Sphere => Sphere::gen(),
                    ModelType::Cube => Cube::gen(),
                };
                let (vertex_buffer, index_buffer) = match model_type {
                    ModelType::Plane => Plane::to_buffer(vertices, indices.clone(), context),
                    ModelType::Sphere => Sphere::to_buffer(vertices, indices.clone(), context),
                    ModelType::Cube => Cube::to_buffer(vertices, indices.clone(), context),
                };
                ModelBuffers {
                    vertex_buffer,
                    index_buffer,
                    index_count: indices.len() as u32,
                }
            })
            .collect()
    }
}
