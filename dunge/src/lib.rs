mod camera;
mod canvas;
pub mod color;
mod context;
mod frame;
mod instance;
mod layout;
mod r#loop;
mod mesh;
mod pipline;
mod render;
mod shader_consts;
mod size;
mod storage;
mod texture;
mod time;
pub mod transform;

pub mod input {
    pub use crate::r#loop::{Input, Key, Mouse, PressedKeys, PressedKeysIterator};
}

#[cfg(not(target_arch = "wasm32"))]
pub use crate::canvas::make_window;

#[cfg(target_arch = "wasm32")]
pub use crate::canvas::from_element;

pub use crate::{
    camera::{Orthographic, Perspective, View},
    canvas::{Canvas, InitialState, WindowMode},
    context::{Context, FrameParameters, Limits},
    frame::Frame,
    layout::{ColorVertex, TextureVertex, Vertex},
    mesh::MeshData,
    r#loop::{Error, Loop},
    render::{InstanceHandle, MeshHandle, TextureHandle},
    texture::{FrameFilter, TextureData},
};
