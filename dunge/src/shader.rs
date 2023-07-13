//! Shader components.

pub use dunge_shader::{
    Color, LightSpaces, SourceArray, SourceArrays, SourceKind, SpaceKind, TexturesNumber,
    View as ShaderView,
};

use {
    crate::vertex::{Vertex, VertexInfo},
    dunge_shader::{Dimension, Fragment, Scheme, Vertex as SchemeVertex},
};

/// The trait defines the shader information.
///
/// This trait has no methods, instead it defines static information about
/// the shader and is used to generate it. To create a shader scheme, call
/// the [`create_scheme`](crate::Context::create_scheme) function on
/// the [context](crate::Context).
pub trait Shader {
    /// Mesh vertex type description.
    type Vertex: Vertex;

    /// Determines whether to use the camera in the shader.
    const VIEW: ShaderView = ShaderView::None;

    /// Determines whether to use the ambient color.
    const AMBIENT: bool = false;

    /// Specifies a static color.
    const STATIC_COLOR: Option<Color> = None;

    /// Specifies textures.
    const TEXTURES: TexturesNumber = TexturesNumber::N0;

    /// Determines whether to use light sources.
    const SOURCES: SourceArrays = SourceArrays::EMPTY;

    /// Determines whether to use light spaces.
    const SPACES: LightSpaces = LightSpaces::EMPTY;

    /// Determines whether to use color instancing.
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
        textures: S::TEXTURES,
        source_arrays: S::SOURCES,
        light_spaces: S::SPACES,
        instance_colors: S::INSTANCE_COLORS,
    }
}

pub(crate) struct ShaderInfo {
    instances: Instances,
    has_camera: bool,
    has_ambient: bool,
    maps: usize,
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
            maps: S::TEXTURES.len(),
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

    pub const fn texture_maps(&self) -> usize {
        self.maps
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
        self.texture_maps() > 0
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
