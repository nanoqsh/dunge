#![cfg_attr(all(doc, not(doctest)), doc = include_str!("../README.md"))]

pub mod buffer;
pub mod color;
mod context;
mod draw;
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
#[doc(hidden)]
pub mod surface;
pub mod usage;
pub mod value;
pub mod vertex;
pub mod workload;

#[cfg(feature = "winit")]
mod _time;
#[cfg(feature = "winit")]
pub mod _window;
#[cfg(feature = "winit")]
mod el;
#[cfg(feature = "winit")]
mod element;
#[cfg(feature = "winit")]
mod update;

/// The dunge prelude.
pub mod prelude {
    pub use crate::{
        _Frame, Group, Instance, Options, Vertex,
        buffer::{Format, TextureData},
        context::Context,
        mesh::MeshData,
        sl, types,
    };

    #[cfg(feature = "winit")]
    pub use crate::{
        _window::View,
        el::{Control, KeyCode, Then},
    };
}

pub use {
    crate::{
        context::{Context, FailedMakeContext, context},
        draw::{Draw, draw},
        shader::{ComputeShader, RenderShader, Shader},
        state::{_Frame, AsTarget, Options, RenderBuffer, Scheduler, Target},
    },
    dunge_macros::{Group, Instance, Vertex},
    dunge_shader::{group::Group, instance::Instance, sl, types, vertex::Vertex},
    glam,
};

#[cfg(not(target_family = "wasm"))]
pub use crate::runtime::block_on;

#[cfg(all(feature = "winit", not(target_family = "wasm")))]
pub use crate::_window::window;

#[cfg(all(feature = "winit", target_family = "wasm"))]
pub use crate::_window::from_element;

#[cfg(feature = "winit")]
pub use crate::{
    el::{Buttons, Control, Flow, Key, KeyCode, LoopError, Mouse, MouseButton, SmolStr, Then},
    update::{IntoUpdate, Update, make, update, update_with_event, update_with_state},
};
