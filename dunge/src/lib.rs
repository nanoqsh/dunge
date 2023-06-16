mod _shader;
pub mod _vertex;
mod camera;
mod canvas;
pub mod color;
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
mod groups;
pub mod handles;
mod layer;
mod layout;
mod r#loop;
mod mesh;
mod pipeline;
mod postproc;
mod render;
mod resources;
mod screen;
mod shader_data {
    mod ambient;
    mod camera;
    pub(crate) mod globals;
    mod instance;
    mod light;
    mod post;
    mod space;
    mod texture;
    pub(crate) mod textures;

    pub(crate) use self::{
        ambient::AmbientUniform,
        camera::CameraUniform,
        instance::{Instance, InstanceModel},
        light::{Light, SourceModel},
        post::PostShaderData,
        space::{LightSpace, SpaceModel},
        texture::Texture,
    };

    pub use self::{
        light::{LightKind, Source},
        space::{Data as SpaceData, Format as SpaceFormat, Space},
        texture::{Data as TextureData, Error as TextureError},
    };
}
pub mod shader;
mod storage;
mod time;
pub mod topology;
pub mod transform;
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
        camera::{Orthographic, Perspective, View},
        canvas::{Backend, Canvas, CanvasConfig, Device, Error as CanvasError, Selector},
        context::{Context, FrameParameters, Limits, PixelSize},
        error::Error,
        frame::Frame,
        framebuffer::FrameFilter,
        layer::{Builder as LayerBuilder, Layer},
        mesh::{Data as MeshData, Error as MeshError},
        pipeline::{Blend, Compare, DrawMode, ParametersBuilder as LayerParametersBuilder},
        r#loop::Loop,
        shader::Shader,
        shader_data::{
            LightKind, Source, Space, SpaceData, SpaceFormat, TextureData, TextureError,
        },
        vertex::Vertex,
    },
    dunge_macros::Vertex,
    winit,
};
