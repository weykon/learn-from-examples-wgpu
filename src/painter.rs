use super::gfx::GfxContext;
use crate::gfx;
use crate::model;
use crate::studio::AsAny;
use crate::utils::Vertex;
use crate::Game;
use model::create_texels;
use model::create_vertices;
use model::generate_matrix;
use std::any::Any;
use std::borrow::Cow;
use std::sync::Weak;
use wgpu::util::DeviceExt;
use wgpu::Buffer;
use wgpu::PipelineLayout;
use wgpu::Texture;
use wgpu::TextureView;

pub trait Painter {
    fn paint(&mut self, context: &gfx::GfxContext, dt: f32, time: f32);
}
pub(crate) trait Sandy {
    type Extra;
    fn ready(context: &gfx::GfxContext, extra: Self::Extra) -> Self
    where
        Self: Sized;
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
