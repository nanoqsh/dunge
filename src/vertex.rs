use wgpu::{VertexAttribute, VertexBufferLayout};

pub trait Vertex: Sized {
    const ATTRIBS: &'static [VertexAttribute];
}

pub(crate) fn layout<V>() -> VertexBufferLayout<'static>
where
    V: Vertex,
{
    use {
        std::mem,
        wgpu::{BufferAddress, VertexStepMode},
    };

    VertexBufferLayout {
        array_stride: mem::size_of::<V>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: V::ATTRIBS,
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ColorVertex {
    pub pos: [f32; 3],
    pub col: [f32; 3],
}

impl Vertex for ColorVertex {
    const ATTRIBS: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TextureVertex {
    pub pos: [f32; 3],
    pub map: [f32; 2],
}

impl Vertex for TextureVertex {
    const ATTRIBS: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
}
