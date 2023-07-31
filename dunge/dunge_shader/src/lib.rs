mod nodes;
mod out;
mod parts {
    mod ambient;
    mod color;
    mod instance;
    mod post;
    mod sources;
    mod spaces;
    mod textures;
    mod vertex;
    mod view;

    pub(crate) use self::{
        ambient::Ambient,
        instance::{InstanceColorInput, InstanceInput},
        post::Post,
        vertex::{VertexInput, VertexOutput},
    };

    pub use self::{
        color::Color,
        post::Vignette,
        sources::{SourceArray, SourceArrays, SourceBindings, SourceKind},
        spaces::{LightSpaces, SpaceBindings, SpaceKind},
        textures::{TextureBindings, TexturesNumber},
        vertex::{Dimension, Fragment},
        view::View,
    };
}
mod shader;
mod templater;

pub use crate::{parts::*, shader::*};
