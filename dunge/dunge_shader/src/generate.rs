use crate::{
    nodes::{Binding, Location},
    out::Out,
    parts::*,
    templater::Templater,
};

pub fn generate(scheme: Scheme) -> ShaderInfo {
    let Scheme {
        vert,
        view,
        ambient,
        static_color,
    } = scheme;

    let vert_input = VertexInput {
        fragment: vert.fragment,
        pos: vert.dimension,
    };

    let vert_output = VertexOutput {
        fragment: vert.fragment,
        static_color,
        ambient,
        world: true,
    };

    let types = {
        let mut o = Out::new();
        let mut location = Location::new();
        InstanceInput::define_type(&mut location, &mut o);
        vert_input.define_type(&mut location, &mut o);

        let mut location = Location::new();
        vert_output.define_type(&mut location, &mut o);

        view.define_type(&mut o);
        o
    };

    let (groups, layout) = {
        let mut o = Out::new();
        let layout = Layout::new(
            |group, o| {
                let mut binding = Binding::with_group(group);
                Globals {
                    post_data: None,
                    camera: view.declare_group(&mut binding, o),
                    ambient: ambient.then(|| Ambient::declare_group(&mut binding, o)),
                }
            },
            |group, o| {
                let mut binding = Binding::with_group(group);
                Textures {
                    map: vert
                        .fragment
                        .vertex_texture
                        .then(|| Texture::declare_group(&mut binding, o)),
                }
            },
            &mut o,
        );

        (o, layout)
    };

    let vertex_out = {
        let mut o = Out::new();
        vert_output.calc_vertex(vert_input, view, &mut o);
        o
    };

    let fragment_col = {
        let mut o = Out::new();
        vert_output.calc_fragment(&mut o);
        o
    };

    ShaderInfo {
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
    pub view: View,
    pub ambient: bool,
    pub static_color: Option<Color>,
}

#[derive(Clone, Copy)]
pub struct Vertex {
    pub dimension: Dimension,
    pub fragment: Fragment,
}

#[must_use]
pub struct ShaderInfo {
    pub layout: Layout,
    pub source: String,
}

impl ShaderInfo {
    pub const VERTEX_ENTRY_POINT: &str = "vs_main";
    pub const FRAGMENT_ENTRY_POINT: &str = "fs_main";

    pub fn postproc(post_data: u32, map: TextureBindings, source: String) -> Self {
        let mut o = Out::new();
        Self {
            layout: Layout::new(
                |group, _| {
                    assert_eq!(group, 0);
                    Globals {
                        post_data: Some(post_data),
                        ..Default::default()
                    }
                },
                |group, _| {
                    assert_eq!(group, 1);
                    Textures { map: Some(map) }
                },
                &mut o,
            ),
            source,
        }
    }
}

pub struct Layout {
    pub globals: Group<Globals>,
    pub textures: Group<Textures>,
}

impl Layout {
    fn new<G, T>(globals: G, textures: T, o: &mut Out) -> Self
    where
        G: FnOnce(u32, &mut Out) -> Globals,
        T: FnOnce(u32, &mut Out) -> Textures,
    {
        let mut num = 0;
        let globals = Group {
            num,
            bindings: globals(num, o),
        };

        if !globals.bindings.is_empty() {
            num += 1;
        }

        let textures = Group {
            num,
            bindings: textures(num, o),
        };

        Self { globals, textures }
    }
}

pub struct Group<T> {
    pub num: u32,
    pub bindings: T,
}

#[derive(Default)]
pub struct Globals {
    pub post_data: Option<u32>,
    pub camera: Option<u32>,
    pub ambient: Option<u32>,
}

impl Globals {
    fn is_empty(&self) -> bool {
        self.post_data.is_none() && self.camera.is_none() && self.ambient.is_none()
    }
}

#[derive(Default)]
pub struct Textures {
    pub map: Option<TextureBindings>,
}

impl Textures {
    fn is_empty(&self) -> bool {
        self.map.is_none()
    }
}
