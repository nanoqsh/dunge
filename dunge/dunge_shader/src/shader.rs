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
    pub textures: TexturesNumber,
    pub source_arrays: SourceArrays,
    pub light_spaces: LightSpaces,
    pub instance_colors: bool,
}

#[derive(Clone, Copy)]
pub struct Vertex {
    pub dimension: Dimension,
    pub fragment: Fragment,
}

pub struct PostScheme {
    pub post_data: u32,
    pub map: TextureBindings,
}

#[must_use]
pub struct Shader {
    pub layout: Layout,
    pub source: String,
}

impl Shader {
    pub const VERTEX_ENTRY_POINT: &str = "vsmain";
    pub const FRAGMENT_ENTRY_POINT: &str = "fsmain";

    pub fn generate(scheme: Scheme) -> Self {
        let Scheme {
            vert,
            view,
            ambient,
            static_color,
            textures,
            source_arrays,
            light_spaces,
            instance_colors,
        } = scheme;

        let vert_input = VertexInput {
            fragment: vert.fragment,
            pos: vert.dimension,
        };

        let vert_output = VertexOutput {
            fragment: vert.fragment,
            static_color,
            ambient,
            textures,
            source_arrays,
            light_spaces,
            instance_colors,
        };

        let instance_color_input = InstanceColorInput::new(instance_colors);
        let instance_color = {
            let mut o = Out::new();
            instance_color_input.define_input(&mut o);
            o
        };

        let types = {
            let mut o = Out::new();
            let mut location = Location::new();
            InstanceInput::define_type(&mut location, &mut o);
            instance_color_input.define_type(&mut location, &mut o);
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
                map: textures.declare_group(binding, o),
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
                .insert("instance_color", &instance_color)
                .insert("vertex_out", &vertex_out)
                .insert("fragment_col", &fragment_col)
                .format(include_str!("../template.wgsl"))
                .expect("generate shader"),
        }
    }

    pub fn postproc(scheme: PostScheme, source: String) -> Self {
        let PostScheme { post_data, map } = scheme;
        let layout = Layout::postproc(post_data, map);
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

    fn postproc(post_data: u32, map: TextureBindings) -> Self {
        Self {
            globals: Group {
                num: 0,
                bindings: Globals {
                    post_data: Some(post_data),
                    ..Default::default()
                },
            },
            textures: Group {
                num: 1,
                bindings: Textures { map },
            },
            lights: Group::default(),
            spaces: Group::default(),
        }
    }
}

#[derive(Default)]
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
    pub map: TextureBindings,
}

impl Textures {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.tmaps.is_empty()
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
