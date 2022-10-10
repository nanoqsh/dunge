mod camera;
mod canvas;
pub mod color;
mod context;
mod frame;
mod instance;
mod layout;
mod r#loop;
mod mesh;
mod render;
mod size;
mod storage;
mod texture;

pub mod input {
    pub use crate::r#loop::{Input, Key, Mouse, PressedKeys, PressedKeysIterator};
}

pub use crate::{
    camera::{Orthographic, Perspective, View},
    canvas::{from_canvas, make_window, Canvas, InitialState, WindowMode},
    context::Context,
    frame::Frame,
    instance::{InstanceData, IntoInstances},
    layout::{ColorVertex, Layout, TextureVertex},
    mesh::MeshData,
    r#loop::{Error, Loop},
    render::{InstanceHandle, MeshHandle, TextureHandle},
    size::Size,
    texture::TextureData,
};
