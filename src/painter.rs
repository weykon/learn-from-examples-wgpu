use model::create_texels;
use model::create_vertices;
use wgpu::util::DeviceExt;
use wgpu::Texture;
use wgpu::TextureView;
use std::borrow::Cow;
use model::generate_matrix;
use crate::gfx;
use crate::model;
use crate::utils::Vertex;
use super::gfx::GfxContext;
use wgpu::Buffer;
use wgpu::PipelineLayout;

pub trait Painter {
    fn paint(&self, context: &gfx::GfxContext);
}
pub(crate) trait Sandy {
    fn ready(context: &gfx::GfxContext) -> Self where Self: Sized;
}

pub(crate) struct VertexBuff {
    pub(crate) vertex_buf: Buffer,
    pub(crate) index_buf: Buffer,
    pub(crate) vertex_size: usize,
    pub(crate) index_count: usize,
}

pub(crate) struct TextureBuff {
    pub(crate) texture: Texture,
    pub(crate) texels: Vec<u8>,
    pub(crate) texture_extent: wgpu::Extent3d,
    pub(crate) texture_view: TextureView,
    pub(crate) size: u32,
}
