mod buffer;
mod canvas;
mod color;
mod context;
mod element;
mod frame;
mod layer;
mod r#loop;
mod mesh;
mod pipeline;
mod posteffect;
mod postproc;
mod render;
mod scheme;
mod screen;
pub mod shader;
mod time;
pub mod topology;
mod transform;
pub mod vertex;
mod view;

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

mod framebuffer {
    mod buffer;
    mod depth_frame;
    mod render_frame;

    pub(crate) use self::buffer::{BufferSize, Framebuffer};
}

pub mod input {
    //! User's input types.

    pub use crate::r#loop::{Input, Key, Keys, KeysIterator, Mouse};
}

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

#[cfg(not(target_arch = "wasm32"))]
pub use crate::canvas::window::{make_window, InitialState, WindowMode};

#[cfg(target_arch = "wasm32")]
pub use crate::canvas::from_element;

#[cfg(target_os = "android")]
pub use crate::canvas::android::from_app;

pub use {
    crate::{
        canvas::{Backend, Canvas, CanvasConfig, Device, Info, Selector},
        color::{Color, Rgb, Rgba},
        context::{Context, FrameParameters, PixelSize},
        frame::Frame,
        input::Input,
        layer::{ActiveLayer, Builder as ActiveLayerBuilder, Layer},
        mesh::{Data as MeshData, Mesh},
        pipeline::{Blend, Compare, DrawMode, LayerBuilder},
        posteffect::{Builder as PostEffectBuilder, PostEffect},
        postproc::FrameFilter,
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
        view::{Orthographic, Perspective, Projection, View, ViewHandle},
    },
    dunge_macros::Vertex,
    glam, winit,
};
