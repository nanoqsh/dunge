use {
    crate::vertex::Vertex,
    dunge_shader::{Color, View},
};

pub trait Shader {
    type Vertex: Vertex;
    const VIEW: View = View::None;
    const BASE_COLOR: Option<Color> = None;
}
