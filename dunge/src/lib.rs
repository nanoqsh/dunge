mod bind_groups;
mod camera;
mod canvas;
pub mod color;
mod context;
mod frame;
mod framebuffer {
    mod buffer;
    mod depth_frame;
    mod render_frame;

    pub(crate) use self::buffer::Framebuffer;
    pub use self::render_frame::FrameFilter;
}
pub mod error;
pub mod handles;
mod layer;
mod layout;
mod r#loop;
mod mesh;
mod pipeline;
mod render;
mod screen;
mod shader;
mod shader_data {
    mod camera;
    mod instance;
    mod light;
    mod post;
    mod space;
    mod texture;

    pub(crate) use self::{
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
pub use crate::canvas::make_window;

#[cfg(target_arch = "wasm32")]
pub use crate::canvas::from_element;

pub use {
    crate::{
        camera::{Orthographic, Perspective, View},
        canvas::{
            Backend, BackendSelector, Canvas, CanvasConfig, Device, Error as CanvasError,
            InitialState, WindowMode,
        },
        context::{Context, FrameParameters, Limits, PixelSize},
        error::Error,
        frame::Frame,
        framebuffer::FrameFilter,
        layer::{Builder as LayerBuilder, Layer},
        mesh::Data as MeshData,
        pipeline::{Blend, Compare, DrawMode, ParametersBuilder as LayerParametersBuilder},
        r#loop::Loop,
        shader_data::{
            LightKind, Source, Space, SpaceData, SpaceFormat, TextureData, TextureError,
        },
    },
    winit,
};
