mod nodes;
mod out;
mod parts {
    mod ambient;
    mod instance;
    mod sources;
    mod spaces;
    mod textures;
    mod vertex;
    mod view;

    pub(crate) use self::{
        ambient::Ambient,
        instance::{InstanceColorInput, InstanceInput},
        vertex::{VertexInput, VertexOutput},
    };

    pub use self::{
        sources::{SourceArray, SourceArrays, SourceBindings, SourceKind},
        spaces::{LightSpaces, SpaceBindings, SpaceKind},
        textures::{TextureBindings, TexturesNumber},
        vertex::{Color, Dimension, Fragment},
        view::View,
    };
}
mod shader;
mod templater;

pub use crate::{parts::*, shader::*};
