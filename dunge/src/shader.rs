use {
    crate::vertex::{Component2D, Vertex},
    dunge_shader::{Color, LightSpaces, SourceArrays, View},
};

pub trait Shader {
    type Vertex: Vertex;
    const VIEW: View = View::None;
    const AMBIENT: bool = false;
    const STATIC_COLOR: Option<Color> = None;
    const SOURCES: SourceArrays = SourceArrays::EMPTY;
    const SPACES: LightSpaces = LightSpaces::EMPTY;
}

pub(crate) struct ShaderInfo {
    pub has_camera: bool,
    pub has_ambient: bool,
    pub has_map: bool,
    pub source_arrays: usize,
    pub light_spaces: LightSpaces,
}

impl ShaderInfo {
    pub const fn new<S>() -> Self
    where
        S: Shader,
    {
        Self {
            has_camera: matches!(S::VIEW, View::Camera),
            has_ambient: S::AMBIENT,
            has_map: <S::Vertex as Vertex>::Texture::OPTIONAL_N_FLOATS.is_some(),
            source_arrays: S::SOURCES.len(),
            light_spaces: S::SPACES,
        }
    }

    pub const fn has_globals(&self) -> bool {
        self.has_camera || self.has_ambient
    }

    pub const fn has_textures(&self) -> bool {
        self.has_map
    }

    pub const fn has_lights(&self) -> bool {
        self.source_arrays > 0
    }

    pub const fn has_spaces(&self) -> bool {
        !self.light_spaces.is_empty()
    }
}
