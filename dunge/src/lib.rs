#![cfg_attr(all(doc, not(doctest)), doc = include_str!("../README.md"))]

pub mod buffer;
pub mod color;
mod context;
pub mod instance;
mod layer;
pub mod mesh;
pub mod render;
mod runtime;
pub mod set;
mod shader;
mod state;
pub mod store;
#[doc(hidden)]
pub mod surface;
pub mod usage;

/// The dunge prelude.
pub mod prelude {
    pub use crate::{
        Bytes, Input, Value, buffer::TextureData, color::ColorExt as _, context::Context, dunge,
        mesh::MeshData, render, sh::sl,
    };
}

pub mod sh {
    pub use dunge_shade::*;
}

pub use {
    crate::{
        context::{Context, FailedMakeContext, context},
        layer::{Blend, Config, Depth, Layer, Topology},
        shader::{RenderShader, Shader},
        state::{AsTarget, Options, RenderBuffer, Scheduler, Target},
    },
    dunge_macro::{Bytes, Input, Value, dunge, render},
    dunge_shade::{
        bytes::Bytes,
        irc::{Input, Value},
    },
};

#[cfg(not(target_family = "wasm"))]
pub use crate::runtime::block_on;
