use {
    crate::vertex::Vertex,
    dunge_shader::{Color, View},
};

pub trait Shader {
    type Vertex: Vertex;
    const VIEW: View = View::None;
    const AMBIENT: bool = false;
    const STATIC_COLOR: Option<Color> = None;
}

pub(crate) const fn has_globals<S>() -> bool
where
    S: Shader,
{
    matches!(S::VIEW, View::Camera) || S::AMBIENT
}
