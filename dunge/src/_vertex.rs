//! Vertex types.

use {
    crate::{
        _shader::ShaderType,
        layout::{Plain, _Layout},
    },
    wgpu::{VertexAttribute, VertexStepMode},
};

/// A trait that describes a vertex.
pub trait _Vertex: _Layout + ShaderType {}

/// Vertex for drawing colored triangles.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ColorVertex {
    pub pos: [f32; 3],
    pub col: [f32; 3],
}

unsafe impl Plain for ColorVertex {}

impl _Layout for ColorVertex {
    const ATTRIBS: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    const VERTEX_STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
}

impl _Vertex for ColorVertex {}

/// Vertex for drawing textured triangles.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TextureVertex {
    pub pos: [f32; 3],
    pub map: [f32; 2],
}

unsafe impl Plain for TextureVertex {}

impl _Layout for TextureVertex {
    const ATTRIBS: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    const VERTEX_STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
}

impl _Vertex for TextureVertex {}

/// Vertex for drawing flat sprites.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FlatVertex {
    pub pos: [f32; 2],
    pub map: [f32; 2],
}

unsafe impl Plain for FlatVertex {}

impl _Layout for FlatVertex {
    const ATTRIBS: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    const VERTEX_STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
}

impl _Vertex for FlatVertex {}
