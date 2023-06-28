use crate::{
    nodes::{Binding, Location},
    out::Out,
    parts::*,
    templater::Templater,
};

#[derive(Clone, Copy)]
pub struct Scheme {
    pub vert: Vertex,
    pub view: View,
    pub ambient: bool,
    pub static_color: Option<Color>,
    pub source_arrays: SourceArrays,
    pub light_spaces: LightSpaces,
}

#[derive(Clone, Copy)]
pub struct Vertex {
    pub dimension: Dimension,
    pub fragment: Fragment,
}

#[must_use]
pub struct Shader {
    pub layout: Layout,
    pub source: String,
}

impl Shader {
    pub const VERTEX_ENTRY_POINT: &str = "vs_main";
    pub const FRAGMENT_ENTRY_POINT: &str = "fs_main";

    pub fn generate(scheme: Scheme) -> Self {
        let Scheme {
            vert,
            view,
            ambient,
            static_color,
            source_arrays,
            light_spaces,
        } = scheme;

        let vert_input = VertexInput {
            fragment: vert.fragment,
            pos: vert.dimension,
        };

        let vert_output = VertexOutput {
            fragment: vert.fragment,
            static_color,
            ambient,
            source_arrays,
            light_spaces,
        };

        let types = {
            let mut o = Out::new();
            let mut location = Location::new();
            InstanceInput::define_type(&mut location, &mut o);
            vert_input.define_type(&mut location, &mut o);

            let mut location = Location::new();
            vert_output.define_type(&mut location, &mut o);
            view.define_type(&mut o);
            source_arrays.define_type(&mut o);
            light_spaces.define_type(&mut o);
            o
        };

        let (layout, groups) = Layout::new(
            |binding, o| Globals {
                post_data: None,
                camera: view.declare_group(binding, o),
                ambient: ambient.then(|| Ambient::declare_group(binding, o)),
            },
            |binding, o| Textures {
                map: vert
                    .fragment
                    .vertex_texture
                    .then(|| Texture::declare_group(binding, o)),
            },
            |binding, o| Lights {
                source_arrays: source_arrays.declare_group(binding, o),
            },
            |binding, o| Spaces {
                light_spaces: light_spaces.declare_group(binding, o),
            },
        );

        let vertex_out = {
            let mut o = Out::new();
            vert_output.calc_vertex(&vert_input, view, &mut o);
            o
        };

        let fragment_col = {
            let mut o = Out::new();
            vert_output.calc_fragment(&mut o);
            o
        };

        Self {
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

    pub fn postproc(post_data: u32, map: TextureBindings, source: String) -> Self {
        let (layout, _) = Layout::new(
            |binding, _| {
                assert_eq!(binding.group(), 0, "expected group 0 for the globals");
                Globals {
                    post_data: Some(post_data),
                    ..Default::default()
                }
            },
            |binding, _| {
                assert_eq!(binding.group(), 1, "expected group 1 for the textures");
                Textures { map: Some(map) }
            },
            |_, _| Lights::default(),
            |_, _| Spaces::default(),
        );

        Self { layout, source }
    }
}

pub struct Layout {
    pub globals: Group<Globals>,
    pub textures: Group<Textures>,
    pub lights: Group<Lights>,
    pub spaces: Group<Spaces>,
}

impl Layout {
    fn new<G, T, L, S>(globals: G, textures: T, lights: L, spaces: S) -> (Self, Out)
    where
        G: FnOnce(&mut Binding, &mut Out) -> Globals,
        T: FnOnce(&mut Binding, &mut Out) -> Textures,
        L: FnOnce(&mut Binding, &mut Out) -> Lights,
        S: FnOnce(&mut Binding, &mut Out) -> Spaces,
    {
        let mut o = Out::new();
        let mut num = 0;
        let globals = Group {
            num,
            bindings: globals(&mut Binding::with_group(num), &mut o),
        };

        if !globals.bindings.is_empty() {
            num += 1;
        }

        let textures = Group {
            num,
            bindings: textures(&mut Binding::with_group(num), &mut o),
        };

        if !textures.bindings.is_empty() {
            num += 1;
        }

        let lights = Group {
            num,
            bindings: lights(&mut Binding::with_group(num), &mut o),
        };

        if !lights.bindings.is_empty() {
            num += 1;
        }

        let spaces = Group {
            num,
            bindings: spaces(&mut Binding::with_group(num), &mut o),
        };

        (
            Self {
                globals,
                textures,
                lights,
                spaces,
            },
            o,
        )
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

#[derive(Default)]
pub struct Lights {
    pub source_arrays: Vec<SourceBindings>,
}

impl Lights {
    fn is_empty(&self) -> bool {
        self.source_arrays.is_empty()
    }
}

#[derive(Default)]
pub struct Spaces {
    pub light_spaces: SpaceBindings,
}
