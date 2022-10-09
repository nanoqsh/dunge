mod camera;
mod canvas;
pub mod color;
mod context;
mod frame;
mod r#loop;
mod mesh;
mod texture;
mod vertex;

pub use crate::{
    camera::{Orthographic, Perspective, View},
    canvas::{from_canvas, make_window, Canvas, InitialState, WindowMode},
    context::{Context, Error, MeshHandle, TextureHandle},
    frame::Frame,
    mesh::MeshData,
    r#loop::Loop,
    texture::TextureData,
    vertex::{ColorVertex, TextureVertex},
};

pub mod input {
    pub use crate::r#loop::{Input, Mouse};
}

use std::num::NonZeroU32;

pub type Size = (NonZeroU32, NonZeroU32);
