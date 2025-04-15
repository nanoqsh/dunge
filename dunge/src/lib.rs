#![cfg_attr(all(doc, not(doctest)), doc = include_str!("../README.md"))]

pub mod color;
mod context;
mod draw;
mod format;
pub mod group;
pub mod instance;
pub mod layer;
pub mod mesh;
pub mod render;
mod runtime;
pub mod set;
mod shader;
mod state;
pub mod storage;
pub mod texture;
pub mod uniform;
pub mod value;
pub mod vertex;
pub mod workload;

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
        Format, Frame, Group, Instance, Options, Vertex, context::Context, mesh::MeshData, sl,
        texture::TextureData, types,
    };

    #[cfg(feature = "winit")]
    pub use crate::{
        el::{Control, KeyCode, Then},
        window::View,
    };
}

pub use {
    crate::{
        context::{Context, FailedMakeContext, context},
        draw::{Draw, draw},
        format::Format,
        shader::{ComputeShader, RenderShader, Shader},
        state::{AsTarget, Frame, Options, RenderBuffer, Scheduler, Target},
    },
    dunge_macros::{Group, Instance, Vertex},
    dunge_shader::{group::Group, instance::Instance, sl, types, vertex::Vertex},
    glam,
};

#[cfg(not(target_family = "wasm"))]
pub use crate::runtime::block_on;

#[cfg(all(feature = "winit", not(target_family = "wasm")))]
pub use crate::window::window;

#[cfg(all(feature = "winit", target_family = "wasm"))]
pub use crate::window::from_element;

#[cfg(feature = "winit")]
pub use crate::{
    el::{Buttons, Control, Flow, Key, KeyCode, LoopError, Mouse, MouseButton, SmolStr, Then},
    update::{IntoUpdate, Update, make, update, update_with_event, update_with_state},
};
