//! Vertex types.

use {
    crate::{
        _layout::{Plain, _Layout},
        _shader::ShaderType,
    },
    wgpu::{VertexAttribute, VertexStepMode},
};

/// A trait that describes a vertex.
pub trait _Vertex: _Layout + ShaderType {}

/// Vertex for drawing colored triangles.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct _ColorVertex {
    pub pos: [f32; 3],
    pub col: [f32; 3],
}

unsafe impl Plain for _ColorVertex {}

impl _Layout for _ColorVertex {
    const ATTRIBS: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    const VERTEX_STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
}

impl _Vertex for _ColorVertex {}

/// Vertex for drawing textured triangles.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct _TextureVertex {
    pub pos: [f32; 3],
    pub map: [f32; 2],
}

unsafe impl Plain for _TextureVertex {}

impl _Layout for _TextureVertex {
    const ATTRIBS: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    const VERTEX_STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
}

impl _Vertex for _TextureVertex {}

/// Vertex for drawing flat sprites.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct _FlatVertex {
    pub pos: [f32; 2],
    pub map: [f32; 2],
}

unsafe impl Plain for _FlatVertex {}

impl _Layout for _FlatVertex {
    const ATTRIBS: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    const VERTEX_STEP_MODE: VertexStepMode = VertexStepMode::Vertex;
}

impl _Vertex for _FlatVertex {}
