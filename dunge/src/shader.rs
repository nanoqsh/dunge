use {crate::vertex::Vertex, dunge_shader::Color};

pub trait Shader {
    type Vertex: Vertex;
    const BASE_COLOR: Option<Color>;
}

pub trait View: Shader {}
