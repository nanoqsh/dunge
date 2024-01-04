pub mod bind;
pub mod color;
pub mod context;
pub mod draw;
pub mod group;
pub mod layer;
pub mod mesh;
pub mod shader;
pub mod state;
pub mod texture;
#[allow(dead_code)]
mod time;
pub mod vertex;

pub use {
    dunge_macros::Vertex,
    dunge_shader::{group::Group, sl, types, vertex::Vertex},
    glam,
};
