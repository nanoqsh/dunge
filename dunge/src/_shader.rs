use {
    crate::{
        _vertex::{ColorVertex, FlatVertex, TextureVertex},
        layout::_Layout,
        shader_data::InstanceModel,
    },
    wgpu::VertexBufferLayout,
};

pub(crate) const COLOR_GLOBALS_GROUP: u32 = 0;
pub(crate) const COLOR_SOURCES_GROUP: u32 = 1;
pub(crate) const FLAT_TEXTURE_GROUP: u32 = 0;
pub(crate) const POST_DATA_GROUP: u32 = 0;
pub(crate) const POST_TEXTURE_GROUP: u32 = 1;
pub(crate) const TEXTURED_GLOBALS_GROUP: u32 = 0;
pub(crate) const TEXTURED_TEXTURE_GROUP: u32 = 1;
pub(crate) const TEXTURED_SOURCES_GROUP: u32 = 2;
pub(crate) const TEXTURED_SPACE_GROUP: u32 = 3;

pub(crate) const CAMERA_BINDING: u32 = 0;
pub(crate) const TDIFF_BINDING: u32 = 0;
pub(crate) const SDIFF_BINDING: u32 = 1;
pub(crate) const DATA_BINDING: u32 = 0;
pub(crate) const AMBIENT_BINDING: u32 = 0;
pub(crate) const SOURCES_BINDING: u32 = 1;
pub(crate) const SPACES_BINDING: u32 = 0;
pub(crate) const SPACE0_TDIFF_BINDING: u32 = 1;
pub(crate) const SPACE1_TDIFF_BINDING: u32 = 2;
pub(crate) const SPACE2_TDIFF_BINDING: u32 = 3;
pub(crate) const SPACE3_TDIFF_BINDING: u32 = 4;
pub(crate) const SPACE_SDIFF_BINDING: u32 = 5;

pub(crate) const MAX_N_SOURCES: u32 = 64;
pub(crate) const MAX_N_SPACES: u32 = 4;

pub(crate) const VERTEX_BUFFER_SLOT: u32 = 0;
pub(crate) const INSTANCE_BUFFER_SLOT: u32 = 1;

#[derive(Clone, Copy)]
pub(crate) enum _Shader {
    Color,
    Flat,
    Post,
    Textured,
}

impl _Shader {
    pub const fn source(self) -> &'static str {
        match self {
            Self::Color => concat!(
                include_str!("shaders/light.wgsl"),
                include_str!("shaders/color.wgsl"),
            ),
            Self::Flat => include_str!("shaders/flat.wgsl"),
            Self::Post => include_str!("shaders/post.wgsl"),
            Self::Textured => concat!(
                include_str!("shaders/light.wgsl"),
                include_str!("shaders/space.wgsl"),
                include_str!("shaders/textured.wgsl"),
            ),
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
        [_layout::<ColorVertex>(), _layout::<InstanceModel>()];

    const FLAT_BUFFERS: [VertexBufferLayout<'static>; 2] =
        [_layout::<FlatVertex>(), _layout::<InstanceModel>()];

    const POST_BUFFERS: [VertexBufferLayout<'static>; 0] = [];

    const TEXTURED_BUFFERS: [VertexBufferLayout<'static>; 2] =
        [_layout::<TextureVertex>(), _layout::<InstanceModel>()];
}

pub struct ShaderValue(_Shader);

impl ShaderValue {
    pub(crate) fn into_inner(self) -> _Shader {
        let Self(value) = self;
        value
    }
}

/// Getting a shader from a vertex type
pub trait ShaderType {
    const VALUE: ShaderValue;
}

impl ShaderType for ColorVertex {
    const VALUE: ShaderValue = ShaderValue(_Shader::Color);
}

impl ShaderType for FlatVertex {
    const VALUE: ShaderValue = ShaderValue(_Shader::Flat);
}

impl ShaderType for TextureVertex {
    const VALUE: ShaderValue = ShaderValue(_Shader::Textured);
}

const fn _layout<V>() -> VertexBufferLayout<'static>
where
    V: _Layout,
{
    use {std::mem, wgpu::BufferAddress};

    VertexBufferLayout {
        array_stride: mem::size_of::<V>() as BufferAddress,
        step_mode: V::VERTEX_STEP_MODE,
        attributes: V::ATTRIBS,
    }
}