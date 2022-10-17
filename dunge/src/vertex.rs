use {
    crate::layout::{Layout, Plain},
    wgpu::{VertexAttribute, VertexStepMode},
};

/// The vertex type.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VertexType {
    /// Use the [`TextureVertex`] for drawing.
    Texture,

    /// Use the [`ColorVertex`] for drawing.
    Color,
}

/// A trait that describes a vertex.
pub trait Vertex: Layout {
    const TYPE: VertexType;
}

/// Vertex for drawing colored triangles.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ColorVertex {
    pub pos: [f32; 3],
    pub col: [f32; 3],
}

unsafe impl Plain for ColorVertex {}

impl Layout for ColorVertex {
    const ATTRIBS: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    const VERTEX_STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
}

impl Vertex for ColorVertex {
    const TYPE: VertexType = VertexType::Color;
}

/// Vertex for drawing textured triangles.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TextureVertex {
    pub pos: [f32; 3],
    pub map: [f32; 2],
}

unsafe impl Plain for TextureVertex {}

impl Layout for TextureVertex {
    const ATTRIBS: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    const VERTEX_STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
}

impl Vertex for TextureVertex {
    const TYPE: VertexType = VertexType::Texture;
}
