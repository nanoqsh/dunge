mod buffer;
mod camera;
mod canvas;
mod color;
mod context;
mod frame;
mod framebuffer {
    mod buffer;
    mod depth_frame;
    mod render_frame;

    pub(crate) use self::buffer::{BufferSize, Framebuffer, Parameters as FrameParameters};
    pub use self::render_frame::FrameFilter;
}
pub mod error;
pub mod handles;
mod layer;
mod r#loop;
mod mesh;
mod pipeline;
mod postproc;
mod render;
mod resources;
mod scheme;
mod screen;
mod shader_data {
    mod ambient;
    pub(crate) mod globals;
    mod instance;
    mod len;
    pub(crate) mod lights;
    mod post;
    mod source;
    mod space;
    pub(crate) mod spaces;
    mod texture;
    pub(crate) mod textures;

    pub(crate) use self::post::PostShaderData;

    pub use self::{
        globals::{Builder as GlobalsBuilder, Globals},
        instance::{Instance, Model},
        source::Source,
        space::{Data as SpaceData, Format as SpaceFormat, Space},
        texture::{Data as TextureData, Error as TextureError},
    };
}
pub mod shader;
mod storage;
mod time;
pub mod topology;
mod transform;
pub mod vertex;

pub mod input {
    //! User's input types.
    pub use crate::r#loop::{Input, Key, Keys, KeysIterator, Mouse};
}

#[cfg(not(target_arch = "wasm32"))]
pub use crate::canvas::window::{make_window, InitialState, WindowMode};

#[cfg(target_arch = "wasm32")]
pub use crate::canvas::from_element;

#[cfg(target_os = "android")]
pub use crate::canvas::android::from_app;

pub use {
    crate::{
        camera::{Orthographic, Perspective, Projection, View},
        canvas::{Backend, Canvas, CanvasConfig, Device, Error as CanvasError, Selector},
        color::{Color, Rgb, Rgba},
        context::{Context, FrameParameters, Limits, PixelSize},
        error::Error,
        frame::Frame,
        framebuffer::FrameFilter,
        layer::{ActiveLayer, Builder as LayerBuilder, Layer},
        mesh::{Data as MeshData, Error as MeshError, Mesh},
        pipeline::{Blend, Compare, DrawMode, ParametersBuilder as LayerParametersBuilder},
        r#loop::Loop,
        scheme::ShaderScheme,
        shader_data::{
            Globals, GlobalsBuilder, Instance, Model, Source, Space, SpaceData, SpaceFormat,
            TextureData, TextureError,
        },
        transform::Transform,
        vertex::Vertex,
    },
    dunge_macros::Vertex,
    winit,
};
