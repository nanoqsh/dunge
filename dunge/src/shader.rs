pub(crate) use shader_type::{ShaderType, ShaderValue};

use {
    crate::{
        layout::{InstanceModel, Layout},
        vertex::{ColorVertex, FlatVertex, TextureVertex},
    },
    wgpu::VertexBufferLayout,
};

mod shader_type {
    use crate::shader::Shader;

    pub struct ShaderValue(pub(crate) Shader);

    /// Getting a shader from a vertex type
    pub trait ShaderType {
        const SHADER: ShaderValue;
    }
}

pub(crate) const COLOR_CAMERA_GROUP: u32 = 0;
pub(crate) const COLOR_CAMERA_BINDING: u32 = 0;

pub(crate) const _FLAT_TDIFF_GROUP: u32 = 0;
pub(crate) const FLAT_TDIFF_BINDING: u32 = 0;
pub(crate) const FLAT_SDIFF_GROUP: u32 = 0;
pub(crate) const FLAT_SDIFF_BINDING: u32 = 1;

pub(crate) const POST_DATA_GROUP: u32 = 0;
pub(crate) const POST_DATA_BINDING: u32 = 0;
pub(crate) const POST_TDIFF_GROUP: u32 = 1;
pub(crate) const POST_TDIFF_BINDING: u32 = 0;
pub(crate) const _POST_SDIFF_GROUP: u32 = 1;
pub(crate) const POST_SDIFF_BINDING: u32 = 1;

pub(crate) const TEXTURED_CAMERA_GROUP: u32 = 0;
pub(crate) const TEXTURED_CAMERA_BINDING: u32 = 0;
pub(crate) const _TEXTURED_TDIFF_GROUP: u32 = 1;
pub(crate) const TEXTURED_TDIFF_BINDING: u32 = 0;
pub(crate) const TEXTURED_SDIFF_GROUP: u32 = 1;
pub(crate) const TEXTURED_SDIFF_BINDING: u32 = 1;

pub(crate) const VERTEX_BUFFER_SLOT: u32 = 0;
pub(crate) const INSTANCE_BUFFER_SLOT: u32 = 1;

#[derive(Clone, Copy)]
pub(crate) enum Shader {
    Color,
    Flat,
    Post,
    Textured,
}

impl Shader {
    pub const fn from_vertex_type<V>() -> Self
    where
        V: ShaderType,
    {
        let ShaderValue(shader) = V::SHADER;
        shader
    }

    pub const fn source(self) -> &'static str {
        match self {
            Self::Color => include_str!("shaders/color.wgsl"),
            Self::Flat => include_str!("shaders/flat.wgsl"),
            Self::Post => include_str!("shaders/post.wgsl"),
            Self::Textured => include_str!("shaders/textured.wgsl"),
        }
    }

    pub const fn buffers(self) -> &'static [VertexBufferLayout<'static>] {
        match self {
            Self::Color => &Self::COLOR_BUFFERS,
            Self::Flat => &Self::FLAT_BUFFERS,
            Self::Post => &Self::POST_BUFFERS,
            Self::Textured => &Self::TEXTURED_BUFFERS,
        }
    }

    const COLOR_BUFFERS: [VertexBufferLayout<'static>; 2] =
        [layout::<ColorVertex>(), layout::<InstanceModel>()];

    const FLAT_BUFFERS: [VertexBufferLayout<'static>; 2] =
        [layout::<FlatVertex>(), layout::<InstanceModel>()];

    const POST_BUFFERS: [VertexBufferLayout<'static>; 0] = [];

    const TEXTURED_BUFFERS: [VertexBufferLayout<'static>; 2] =
        [layout::<TextureVertex>(), layout::<InstanceModel>()];
}

impl ShaderType for ColorVertex {
    const SHADER: ShaderValue = ShaderValue(Shader::Color);
}

impl ShaderType for FlatVertex {
    const SHADER: ShaderValue = ShaderValue(Shader::Flat);
}

impl ShaderType for TextureVertex {
    const SHADER: ShaderValue = ShaderValue(Shader::Textured);
}

const fn layout<V>() -> VertexBufferLayout<'static>
where
    V: Layout,
{
    use {std::mem, wgpu::BufferAddress};

    VertexBufferLayout {
        array_stride: mem::size_of::<V>() as BufferAddress,
        step_mode: V::VERTEX_STEP_MODE,
        attributes: V::ATTRIBS,
    }
}
