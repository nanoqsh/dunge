use {
    crate::vertex::Vertex,
    dunge_shader::{Camera, Color},
};

pub trait Shader {
    type Vertex: Vertex;
    const CAMERA: Camera = Camera::None;
    const COLOR: Option<Color> = None;
}
