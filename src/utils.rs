pub mod models;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct Vertex {
    pub(crate) _pos: [f32; 4],
    pub(crate) _tex_coord: [f32; 2],
}

pub(crate) fn vertex(pos: [i8; 3], tc: [i8; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        _tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}

impl Vertex {
    pub fn to_f32_array(&self) -> [f32; 6] {
        [
            self._pos[0],
            self._pos[1],
            self._pos[2],
            self._tex_coord[0],
            self._tex_coord[1],
            self._pos[3], // 将w分量放在最后
        ]
    }
}
