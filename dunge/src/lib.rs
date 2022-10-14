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
mod size;
mod storage;
mod texture;
mod time;

pub mod input {
    pub use crate::r#loop::{Input, Key, Mouse, PressedKeys, PressedKeysIterator};
}

pub mod rotation {
    pub use crate::instance::{AxisAngle, Identity, Inversed, Quat, Rotation};
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
    instance::InstanceData,
    layout::{ColorVertex, Layout, TextureVertex},
    mesh::MeshData,
    r#loop::{Error, Loop},
    render::{InstanceHandle, MeshHandle, TextureHandle},
    texture::{FrameFilter, TextureData},
};
