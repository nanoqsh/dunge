mod camera;
mod canvas;
pub mod color;
mod context;
mod frame;
mod r#loop;
mod mesh;
mod render;
mod size;
mod texture;
mod vertex;

pub mod input {
    pub use crate::r#loop::{Input, Mouse};
}

pub use crate::{
    camera::{Orthographic, Perspective, View},
    canvas::{from_canvas, make_window, Canvas, InitialState, WindowMode},
    context::Context,
    frame::Frame,
    mesh::MeshData,
    r#loop::Loop,
    render::{MeshHandle, TextureHandle},
    size::Size,
    texture::TextureData,
    vertex::{ColorVertex, TextureVertex, Vertex},
};

#[derive(Debug)]
pub enum Error {
    /// Returns when a rendered resourse not found.
    ResourceNotFound,
}
