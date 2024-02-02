pub mod bind;
pub mod color;
mod context;
mod draw;
mod format;
pub mod group;
mod init;
pub mod instance;
pub mod layer;
pub mod mesh;
mod shader;
mod state;
pub mod texture;
pub mod uniform;
pub mod vertex;

#[cfg(feature = "winit")]
mod el;
#[cfg(feature = "winit")]
mod time;
#[cfg(feature = "winit")]
mod update;
#[cfg(feature = "winit")]
pub mod window;

pub mod prelude {
    pub use crate::{context::Context, shader::Shader, sl, types, Frame, Group, Instance, Vertex};

    #[cfg(feature = "winit")]
    pub use crate::el::{Control, KeyCode, Then};
}

pub use {
    crate::{
        context::{Context, Error},
        draw::{draw, Draw},
        format::Format,
        init::context,
        state::{Frame, Options},
    },
    dunge_macros::{Group, Instance, Vertex},
    dunge_shader::{group::Group, instance::Instance, sl, types, vertex::Vertex},
    glam,
};

#[cfg(feature = "winit")]
pub use crate::{
    el::{Control, Flow, Key, KeyCode, LoopError, SmolStr, Then},
    init::window,
    update::{update, update_with_state, Update},
};
