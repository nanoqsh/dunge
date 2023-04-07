mod camera;
mod canvas;
pub mod color;
mod context;
mod depth_frame;
mod frame;
pub mod handles;
mod instance;
mod layer;
mod layout;
mod r#loop;
mod mesh;
mod pipeline;
mod render;
mod render_frame;
mod screen;
mod shader;
mod shader_data {
    mod camera;
    mod light;
    mod post;

    pub(crate) use self::{camera::CameraUniform, light::Light, post::PostShaderData};
}
mod storage;
mod texture;
mod time;
pub mod topology;
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
    context::{Context, FrameParameters, Limits, PixelSize},
    frame::Frame,
    layer::{Builder as LayerBuilder, Layer},
    mesh::Data as MeshData,
    pipeline::{Blend, Compare, DrawMode, ParametersBuilder as LayerParametersBuilder},
    r#loop::{Error, Loop},
    render_frame::FrameFilter,
    texture::Data as TextureData,
};
