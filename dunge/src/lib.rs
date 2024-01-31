pub mod bind;
pub mod color;
pub mod context;
pub mod draw;
mod format;
pub mod group;
mod init;
pub mod instance;
pub mod layer;
pub mod mesh;
pub mod shader;
mod state;
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

pub mod prelude {
    pub use crate::{context::Context, draw, sl, types, update, Frame, Group, Instance, Vertex};

    #[cfg(feature = "winit")]
    pub use crate::el::{Control, KeyCode, Then};
}

pub use {
    crate::{
        format::Format,
        init::context,
        state::{Frame, Options},
    },
    dunge_macros::{Group, Instance, Vertex},
    dunge_shader::{group::Group, instance::Instance, sl, types, vertex::Vertex},
    glam,
};

#[cfg(feature = "winit")]
pub use crate::init::window;
