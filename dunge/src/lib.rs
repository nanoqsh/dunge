pub mod bind;
pub mod color;
pub mod context;
pub mod draw;
pub mod format;
pub mod group;
mod init;
pub mod layer;
pub mod mesh;
pub mod shader;
pub mod state;
pub mod table;
pub mod texture;
pub mod uniform;
pub mod vertex;

#[cfg(feature = "winit")]
pub mod el;
#[cfg(feature = "winit")]
mod time;
#[cfg(feature = "winit")]
pub mod update;
#[cfg(feature = "winit")]
pub mod window;

pub use {
    crate::{init::context, state::Frame},
    dunge_macros::{Group, Vertex},
    dunge_shader::{group::Group, instance::Instance, sl, types, vertex::Vertex},
    glam,
};

pub mod instance {
    pub use dunge_shader::instance::Projection;
}

#[cfg(feature = "winit")]
pub use {crate::init::window, el::Control};
