pub use dunge_shader::{
    Color, LightSpaces, SourceArray, SourceArrays, SourceKind, SpaceKind, View as ShaderView,
};

use {
    crate::vertex::{Component2D, Vertex, VertexInfo},
    dunge_shader::{Dimension, Fragment, Scheme, Vertex as SchemeVertex},
};

pub trait Shader {
    type Vertex: Vertex;
    const VIEW: ShaderView = ShaderView::None;
    const AMBIENT: bool = false;
    const STATIC_COLOR: Option<Color> = None;
    const SOURCES: SourceArrays = SourceArrays::EMPTY;
    const SPACES: LightSpaces = LightSpaces::EMPTY;
    const INSTANCE_COLORS: bool = false;
}

pub(crate) const fn scheme<S>() -> Scheme
where
    S: Shader,
{
    let vert = VertexInfo::new::<S::Vertex>();
    Scheme {
        vert: SchemeVertex {
            dimension: match vert.dimensions {
                2 => Dimension::D2,
                3 => Dimension::D3,
                _ => unreachable!(),
            },
            fragment: Fragment {
                vertex_color: vert.has_color,
                vertex_texture: vert.has_texture,
            },
        },
        view: S::VIEW,
        static_color: S::STATIC_COLOR,
        ambient: S::AMBIENT,
        source_arrays: S::SOURCES,
        light_spaces: S::SPACES,
        instance_colors: S::INSTANCE_COLORS,
    }
}

pub(crate) struct ShaderInfo {
    instances: Instances,
    has_camera: bool,
    has_ambient: bool,
    has_map: bool,
    source_arrays: usize,
    light_spaces: LightSpaces,
}

impl ShaderInfo {
    pub const fn new<S>() -> Self
    where
        S: Shader,
    {
        Self {
            instances: Instances {
                has_color: S::INSTANCE_COLORS,
            },
            has_camera: matches!(S::VIEW, ShaderView::Camera),
            has_ambient: S::AMBIENT,
            has_map: <S::Vertex as Vertex>::Texture::OPTIONAL_N_FLOATS.is_some(),
            source_arrays: S::SOURCES.len(),
            light_spaces: S::SPACES,
        }
    }

    pub const fn has_camera(&self) -> bool {
        self.has_camera
    }

    pub const fn has_ambient(&self) -> bool {
        self.has_ambient
    }

    pub const fn has_map(&self) -> bool {
        self.has_map
    }

    pub const fn source_arrays(&self) -> usize {
        self.source_arrays
    }

    pub const fn light_spaces(&self) -> LightSpaces {
        self.light_spaces
    }

    pub const fn has_instance_colors(&self) -> bool {
        self.instances.has_color
    }

    pub const fn has_globals(&self) -> bool {
        self.has_camera() || self.has_ambient()
    }

    pub const fn has_textures(&self) -> bool {
        self.has_map()
    }

    pub const fn has_lights(&self) -> bool {
        self.source_arrays() > 0
    }

    pub const fn has_spaces(&self) -> bool {
        !self.light_spaces().is_empty()
    }
}

struct Instances {
    has_color: bool,
}
