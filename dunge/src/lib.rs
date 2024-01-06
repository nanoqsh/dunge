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
pub mod vertex;

#[cfg(feature = "winit")]
mod time;

pub use {
    dunge_macros::{Group, Vertex},
    dunge_shader::{group::Group, sl, types, vertex::Vertex},
    glam,
};
