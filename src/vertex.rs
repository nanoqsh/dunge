use {
    plain::Plain,
    wgpu::{VertexAttribute, VertexBufferLayout},
};

mod plain {
    /// A trait for plain structs which can be safely casted to bytes.
    ///
    /// # Safety
    /// An implementation of this trait assumes all bits of struct can be safely read.
    pub unsafe trait Plain: Sized {}
}

/// The trait describes a vertex.
pub trait Vertex: Plain {
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

/// Don't use this vertex.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ColorVertex {
    pub pos: [f32; 3],
    pub col: [f32; 3],
}

unsafe impl Plain for ColorVertex {}

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

unsafe impl Plain for TextureVertex {}

impl Vertex for TextureVertex {
    const ATTRIBS: &'static [VertexAttribute] =
        &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
}
