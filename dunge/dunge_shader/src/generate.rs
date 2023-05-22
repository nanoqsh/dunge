use {
    crate::{
        elements::*,
        nodes::{Binding, Location},
        out::Out,
        templater::Templater,
    },
    std::borrow::Cow,
};

#[must_use]
pub fn generate(Scheme { vert, camera }: Scheme) -> Shader<'static> {
    let vert_input = VertexInput {
        fragment: &vert.fragment,
        pos: vert.dimension,
    };

    let vert_output = VertexOutput {
        fragment: &vert.fragment,
        world: true,
    };

    let types = {
        let mut o = Out::new();
        let mut location = Location::new();
        vert_input.define_type(&mut location, &mut o);
        InstanceInput::define_type(&mut location, &mut o);

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
            .expect("generate shader")
            .into(),
    }
}

#[derive(Clone, Copy)]
pub struct Scheme {
    pub vert: Vertex,
    pub camera: Camera,
}

#[derive(Clone, Copy)]
pub struct Vertex {
    pub dimension: Dimension,
    pub fragment: Fragment,
}

pub struct Shader<'a> {
    pub layout: Layout,
    pub source: Cow<'a, str>,
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
