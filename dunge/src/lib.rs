#![cfg_attr(all(doc, not(doctest)), doc = include_str!("../README.md"))]

pub mod buffer;
pub mod color;
mod context;
pub mod group;
pub mod instance;
pub mod instance2;
mod layer;
pub mod mesh;
pub mod render;
mod runtime;
pub mod set;
mod shader;
mod state;
pub mod store;
pub mod store2;
#[doc(hidden)]
pub mod surface;
pub mod usage;
mod value;

/// The dunge prelude.
pub mod prelude {
    pub use crate::{
        Bytes, Group, Input, Instance, Value, Vertex, buffer::TextureData, color::ColorExt as _,
        context::Context, dunge, mesh::MeshData, render, sl, types,
    };
}

/// The vertex module.
pub mod vertex {
    pub use dunge_shade_old::vertex::{InputProjection, Projection, verts_as_bytes};
}

pub use {
    crate::{
        context::{Builder, Context, FailedMakeContext, context},
        layer::{Blend, Config, Layer, Polygon, Topology},
        shader::{RenderShader, Shader},
        state::{AsTarget, Options, RenderBuffer, Scheduler, Target},
        value::{ColorValue, StorageValue, UniformValue},
    },
    dunge_macro::{Bytes, Input, Value, dunge, render},
    dunge_macro_old::{Group, Instance, Vertex},
    dunge_shade::{
        bytes::Bytes,
        irc::{Input, Value},
    },
    dunge_shade_old::{group::Group, instance::Instance, sl, types, vertex::Vertex},
};

#[cfg(not(target_family = "wasm"))]
pub use crate::runtime::block_on;
