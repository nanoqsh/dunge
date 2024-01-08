pub mod bind;
pub mod color;
pub mod context;
pub mod draw;
pub mod group;
mod init;
pub mod layer;
pub mod mesh;
pub mod shader;
pub mod state;
pub mod texture;
pub mod vertex;

#[cfg(feature = "winit")]
mod el;
#[cfg(feature = "winit")]
mod time;
#[cfg(feature = "winit")]
pub mod update;
#[cfg(feature = "winit")]
pub mod window;

pub use {
    crate::init::context,
    dunge_macros::{Group, Vertex},
    dunge_shader::{group::Group, sl, types, vertex::Vertex},
    glam,
};

#[cfg(feature = "winit")]
pub use {crate::init::window, el::Control};
