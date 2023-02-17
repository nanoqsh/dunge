mod camera;
mod canvas;
pub mod color;
mod context;
mod depth_frame;
mod frame;
mod instance;
mod layer;
mod layout;
mod r#loop;
mod mesh;
mod render;
mod render_frame;
mod screen;
mod shader_consts;
mod shader_data;
mod storage;
mod texture;
mod time;
pub mod transform;
pub mod vertex;

pub mod input {
    //! User's input types.
    pub use crate::r#loop::{Input, Key, Keys, KeysIterator, Mouse};
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
    layer::{Builder as LayerBuilder, Layer},
    mesh::Data as MeshData,
    r#loop::{Error, Loop},
    render::{InstanceHandle, MeshHandle, TextureHandle, ViewHandle},
    render_frame::FrameFilter,
    texture::Data as TextureData,
};
