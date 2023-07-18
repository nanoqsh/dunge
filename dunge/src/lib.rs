#![doc = include_str!("../../README.md")]

mod buffer;
mod camera;
mod canvas;
mod color;
mod context;
pub mod error {
    //! Error types.

    pub use crate::{
        canvas::Error as CanvasError,
        mesh::{Error as MeshError, UpdateError as MeshUpdateError},
        shader_data::{
            DataError, InvalidInstanceSize, InvalidMapSize, LightsUpdateError, SpacesUpdateError,
        },
    };
}
mod frame;
mod framebuffer {
    mod buffer;
    mod depth_frame;
    mod render_frame;

    pub(crate) use self::buffer::{BufferSize, Framebuffer, Parameters as FrameParameters};
    pub use self::render_frame::FrameFilter;
}

mod layer;
mod r#loop;
mod mesh;
mod pipeline;
mod postproc;
mod render;
mod scheme;
mod screen;
mod shader_data {
    mod ambient;
    mod data;
    pub(crate) mod globals;
    mod instance;
    mod len;
    pub(crate) mod lights;
    mod post;
    mod source;
    mod space;
    pub(crate) mod spaces;
    pub(crate) mod texture;
    pub(crate) mod textures;

    pub(crate) use self::post::PostShaderData;

    pub use self::{
        data::{Error as DataError, Format, SpaceData, TextureData},
        globals::{Builder as GlobalsBuilder, Globals},
        instance::{
            Instance, InstanceColor, InvalidSize as InvalidInstanceSize, ModelColor, ModelTransform,
        },
        lights::{Builder as LightsBuilder, Lights, UpdateError as LightsUpdateError},
        source::Source,
        space::Space,
        spaces::{Builder as SpacesBuilder, Spaces, UpdateError as SpacesUpdateError},
        texture::{InvalidSize as InvalidMapSize, Texture},
        textures::{Builder as TexturesBuilder, Map as MapParameter, Textures},
    };
}
pub mod shader;
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
        canvas::{Backend, Canvas, CanvasConfig, Device, Info, Selector},
        color::{Color, Rgb, Rgba},
        context::{Context, FrameParameters, Limits, PixelSize},
        frame::Frame,
        framebuffer::FrameFilter,
        input::Input,
        layer::{ActiveLayer, Builder as ActiveLayerBuilder, Layer},
        mesh::{Data as MeshData, Mesh},
        pipeline::{Blend, Compare, DrawMode, LayerBuilder},
        r#loop::Loop,
        scheme::Scheme,
        shader::Shader,
        shader_data::{
            Format, Globals, GlobalsBuilder, Instance, InstanceColor, Lights, LightsBuilder,
            MapParameter, ModelColor, ModelTransform, Source, Space, SpaceData, Spaces,
            SpacesBuilder, Texture, TextureData, Textures, TexturesBuilder,
        },
        transform::Transform,
        vertex::Vertex,
    },
    dunge_macros::Vertex,
    winit,
};
