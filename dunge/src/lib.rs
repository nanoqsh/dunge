pub mod bind;
pub mod color;
mod context;
mod draw;
mod format;
pub mod group;
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
mod element;
#[cfg(feature = "winit")]
mod time;
#[cfg(feature = "winit")]
mod update;
#[cfg(feature = "winit")]
pub mod window;

pub mod prelude {
    //! The dunge prelude.

    pub use crate::{
        context::Context, mesh::MeshData, sl, texture::TextureData, types, Format, Frame, Group,
        Instance, Options, Vertex,
    };

    #[cfg(feature = "winit")]
    pub use crate::el::{Control, KeyCode, Then};
}

pub use {
    crate::{
        context::{context, Context, FailedMakeContext},
        draw::{draw, Draw},
        format::Format,
        state::{AsTarget, Frame, Options, RenderBuffer, Target},
    },
    dunge_macros::{Group, Instance, Vertex},
    dunge_shader::{group::Group, instance::Instance, sl, types, vertex::Vertex},
    glam,
};

#[cfg(all(feature = "winit", not(target_arch = "wasm32")))]
pub use crate::window::window;

#[cfg(all(feature = "winit", target_arch = "wasm32"))]
pub use crate::window::from_element;

#[cfg(feature = "winit")]
pub use crate::{
    el::{Buttons, Control, Flow, Key, KeyCode, LoopError, Mouse, MouseButton, SmolStr, Then},
    update::{from_view, update, update_with_event, update_with_state, IntoUpdate, Update},
};
