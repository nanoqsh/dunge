#![cfg_attr(all(doc, not(doctest)), doc = include_str!("../README.md"))]

pub mod buffer;
pub mod color;
pub mod compute;
mod context;
pub mod group;
pub mod instance;
mod layer;
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
mod value;
mod workload;

/// The dunge prelude.
pub mod prelude {
    pub use crate::{
        Group, Instance, Vertex, buffer::TextureData, color::ColorExt as _, context::Context,
        mesh::MeshData, sl, types,
    };
}

/// The vertex module.
pub mod vertex {
    pub use dunge_shader::vertex::{InputProjection, Projection, verts_as_bytes};
}

pub use {
    crate::{
        context::{Builder, Context, FailedMakeContext, context},
        layer::{Blend, Config, Layer, Polygon, Topology},
        shader::{ComputeShader, RenderShader, Shader},
        state::{AsTarget, Options, RenderBuffer, Scheduler, Target},
        value::{UnsizedValue, Value},
        workload::Workload,
    },
    dunge_macros::{Group, Instance, Vertex},
    dunge_shader::{group::Group, instance::Instance, sl, types, vertex::Vertex},
    glam,
};

#[cfg(not(target_family = "wasm"))]
pub use crate::runtime::block_on;
