use crate::{
    elements::*,
    nodes::{Binding, Location},
    out::Out,
    templater::Templater,
};

#[must_use]
pub fn generate(scheme: Scheme) -> Shader {
    let Scheme {
        vert,
        color,
        camera,
    } = scheme;

    let vert_input = VertexInput {
        fragment: vert.fragment,
        pos: vert.dimension,
    };

    let vert_output = VertexOutput {
        fragment: vert.fragment,
        color,
        world: true,
    };

    let types = {
        let mut o = Out::new();
        let mut location = Location::new();
        InstanceInput::define_type(&mut location, &mut o);
        vert_input.define_type(&mut location, &mut o);

        let mut location = Location::new();
        vert_output.define_type(&mut location, &mut o);

        camera.define_type(&mut o);
        o
    };

    let (groups, layout) = {
        let mut o = Out::new();
        let layout = Layout {
            globals: {
                let mut binding = Binding::with_group(Globals::GROUP);
                Globals {
                    camera: camera.declare_group(&mut binding, &mut o),
                }
            },
            textures: {
                let mut binding = Binding::with_group(Textures::GROUP);
                Textures {
                    texture: vert
                        .fragment
                        .vertex_texture
                        .then(|| Texture::declare_group(&mut binding, &mut o)),
                }
            },
        };

        (o, layout)
    };

    let vertex_out = {
        let mut o = Out::new();
        vert_output.calc_vertex(vert_input, camera, &mut o);
        o
    };

    let fragment_col = {
        let mut o = Out::new();
        vert_output.calc_fragment(&mut o);
        o
    };

    Shader {
        layout,
        source: Templater::default()
            .insert("types", &types)
            .insert("groups", &groups)
            .insert("vertex_out", &vertex_out)
            .insert("fragment_col", &fragment_col)
            .format(include_str!("../template.wgsl"))
            .expect("generate shader"),
    }
}

#[derive(Clone, Copy)]
pub struct Scheme {
    pub vert: Vertex,
    pub color: Option<Color>,
    pub camera: Camera,
}

#[derive(Clone, Copy)]
pub struct Vertex {
    pub dimension: Dimension,
    pub fragment: Fragment,
}

pub struct Shader {
    pub layout: Layout,
    pub source: String,
}

impl Shader {
    pub const VERTEX_ENTRY_POINT: &str = "vs_main";
    pub const FRAGMENT_ENTRY_POINT: &str = "fs_main";
}

pub struct Layout {
    pub globals: Globals,
    pub textures: Textures,
}

pub struct Globals {
    pub camera: Option<u32>,
}

impl Globals {
    pub const GROUP: u32 = 0;
}

pub struct Textures {
    pub texture: Option<TextureBindings>,
}

impl Textures {
    pub const GROUP: u32 = 1;
}
