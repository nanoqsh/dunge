//! Vertex types.

use {
    crate::{
        layout::{Layout, Plain},
        shader::ShaderType,
    },
    wgpu::{VertexAttribute, VertexStepMode},
};

/// A trait that describes a vertex.
pub trait Vertex: Layout + ShaderType {}

/// Vertex for drawing colored triangles.
#[allow(clippy::module_name_repetitions)]
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

impl Vertex for ColorVertex {}

/// Vertex for drawing textured triangles.
#[allow(clippy::module_name_repetitions)]
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

impl Vertex for TextureVertex {}

/// Vertex for drawing flat sprites.
#[allow(clippy::module_name_repetitions)]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FlatVertex {
    pub pos: [f32; 2],
    pub map: [f32; 2],
}

unsafe impl Plain for FlatVertex {}

impl Layout for FlatVertex {
    const ATTRIBS: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    const VERTEX_STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
}

impl Vertex for FlatVertex {}

pub trait UsesDepth: Layout {}
impl UsesDepth for ColorVertex {}
impl UsesDepth for TextureVertex {}
