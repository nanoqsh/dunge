use crate::{nodes::*, out::Out};

#[derive(Clone, Copy)]
pub enum Dimension {
    D2,
    D3,
}

#[derive(Clone, Copy)]
pub(crate) struct InstanceInput;

impl InstanceInput {
    pub fn define_type(location: &mut Location, o: &mut Out) {
        const FIELDS: [&str; 4] = ["r0", "r1", "r2", "r3"];

        o.write(Struct {
            name: "InstanceInput",
            fields: FIELDS
                .into_iter()
                .map(|name| Field {
                    location: location.next(),
                    name: Name::Str(name),
                    ty: Type::VEC4,
                })
                .collect(),
        });
    }
}

pub(crate) struct VertexInput {
    pub fragment: Fragment,
    pub pos: Dimension,
}

impl VertexInput {
    pub fn define_type(&self, location: &mut Location, o: &mut Out) {
        let mut fields = vec![Field {
            location: location.next(),
            name: Name::Str("pos"),
            ty: match self.pos {
                Dimension::D2 => Type::VEC2,
                Dimension::D3 => Type::VEC3,
            },
        }];

        if self.fragment.vertex_color {
            fields.push(Field {
                location: location.next(),
                name: Name::Str("col"),
                ty: Type::VEC3,
            });
        }

        if self.fragment.vertex_texture {
            fields.push(Field {
                location: location.next(),
                name: Name::Str("map"),
                ty: Type::VEC2,
            });
        }

        o.write(Struct {
            name: "VertexInput",
            fields,
        });
    }

    pub fn calc_world(&self, o: &mut Out) {
        o.write_str(match self.pos {
            Dimension::D2 => "model * vec4(input.pos, 0., 1.)",
            Dimension::D3 => "model * vec4(input.pos, 1.)",
        });
    }
}

pub(crate) struct VertexOutput {
    pub fragment: Fragment,
    pub static_color: Option<Color>,
    pub ambient: bool,
    pub source_arrays: SourceArrays,
    pub light_spaces: LightSpaces,
}

impl VertexOutput {
    pub fn define_type(&self, location: &mut Location, o: &mut Out) {
        let mut fields = vec![Field {
            location: Location::Position,
            name: Name::Str("pos"),
            ty: Type::VEC4,
        }];

        if self.fragment.vertex_color {
            fields.push(Field {
                location: location.next(),
                name: Name::Str("col"),
                ty: Type::VEC3,
            });
        }

        if self.fragment.vertex_texture {
            fields.push(Field {
                location: location.next(),
                name: Name::Str("map"),
                ty: Type::VEC2,
            });
        }

        if self.has_sources() {
            fields.push(Field {
                location: location.next(),
                name: Name::Str("world"),
                ty: Type::VEC3,
            });
        }

        for (n, ..) in self.light_spaces.enumerate() {
            fields.push(Field {
                location: location.next(),
                name: Name::Num { str: "space", n },
                ty: Type::VEC3,
            });
        }

        o.write(Struct {
            name: "VertexOutput",
            fields,
        });
    }

    pub fn calc_vertex(&self, input: &VertexInput, camera: View, o: &mut Out) {
        o.write_str("let world = ");
        input.calc_world(o);
        o.write_str(";\n");

        o.write_str("    out.pos = ");
        camera.calc_view(o);
        o.write_str(";\n");

        if self.fragment.vertex_color {
            o.write_str("    out.col = input.col;\n");
        }

        if self.fragment.vertex_texture {
            o.write_str("    out.map = input.map;\n");
        }

        if self.has_sources() {
            o.write_str("    out.world = world.xyz;\n");
        }

        for (n, ..) in self.light_spaces.enumerate() {
            o.write_str("    out.space_")
                .write(n)
                .write_str(" = (spaces[")
                .write(n)
                .write_str("].model * world).xzy;\n");
        }
    }

    pub fn calc_fragment(&self, o: &mut Out) {
        if self.fragment.vertex_texture {
            o.write_str(
                "let tex = textureSample(tdiff, sdiff, out.map); \n    \
                if tex.w < 0.95 { \
                    discard; \
                } \n    ",
            );
        }

        let has_light = self.calc_light(o);
        let mut col = o.write_str("col = ").separated(" * ");
        if has_light {
            col.out().write_str("light");
        }

        if let Some(Color { r, g, b }) = self.static_color {
            col.out()
                .write_str("vec3(")
                .write_f32(r)
                .write_str(", ")
                .write_f32(g)
                .write_str(", ")
                .write_f32(b)
                .write_str(")");
        }

        if self.fragment.vertex_color {
            col.out().write_str("out.col");
        }

        if self.fragment.vertex_texture {
            col.out().write_str("tex.rgb");
        }

        col.write_default("vec3(0.)");
        o.write_str(";\n");
    }

    fn calc_light(&self, o: &mut Out) -> bool {
        if !self.ambient && !self.has_sources() && self.light_spaces.is_empty() {
            return false;
        }

        if self.has_sources() {
            o.write_str("\n    ")
                .write_str("var sources = vec3(0.); \n    ");

            for (n, SourceArray { kind, .. }) in self.source_arrays.enumerate() {
                o.write_str("for (var i = 0u; i < ")
                    .write(Name::Num {
                        str: "sources_len",
                        n,
                    })
                    .write_str(".n; i++) {\n        ")
                    .write_str("let src = ")
                    .write(Name::Num {
                        str: "sources_array",
                        n,
                    })
                    .write_str("[i];\n        ")
                    .write_str(
                        "if out.world.x > src.pos.x - src.rad && out.world.x < src.pos.x + src.rad \
                         && out.world.y > src.pos.y - src.rad && out.world.y < src.pos.y + src.rad \
                         && out.world.z > src.pos.z - src.rad && out.world.z < src.pos.z + src.rad \
                         {\n            ",
                    )
                    .write_str("let len = length(out.world - src.pos);\n            ")
                    .write_str("if len < src.rad {\n                ")
                    .write_str("let e = 1. - (len / src.rad);\n                ");

                match kind {
                    SourceKind::Glow => {
                        o.write_str("sources += e * e * src.col");
                    }
                    SourceKind::Gloom if self.ambient => {
                        o.write_str("sources -= e * e * src.col * ambient.rgb");
                    }
                    SourceKind::Gloom => {
                        o.write_str("sources -= e * e * src.col");
                    }
                }

                o.write_str(";\n            ")
                    .write_str("}\n        ")
                    .write_str("}\n    ")
                    .write_str("}\n    ");
            }
        }

        let has_spaces = self.calc_spaces(o);
        let mut light = o.write_str("let light = ").separated(" + ");
        if self.ambient {
            light.out().write_str("ambient.rgb");
        }

        if self.has_sources() {
            light.out().write_str("sources");
        }

        if has_spaces {
            light.out().write_str("space");
        }

        o.write_str(";\n    ");
        true
    }

    pub fn has_sources(&self) -> bool {
        !self.source_arrays.is_empty()
    }

    fn calc_spaces(&self, o: &mut Out) -> bool {
        const NAMES: [&str; 4] = ["space_a", "space_b", "space_c", "space_d"];

        if self.light_spaces.is_empty() {
            return false;
        }

        o.write_str("\n    ")
            .write_str("var space = vec3(0.);\n    ");

        for ((n, kind), name) in self.light_spaces.enumerate().zip(NAMES) {
            kind.calc(name, n, o);
        }

        o.write_str("\n    ");
        true
    }
}

#[derive(Clone, Copy)]
pub struct Fragment {
    pub vertex_color: bool,
    pub vertex_texture: bool,
}

#[derive(Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

pub(crate) struct Texture;

impl Texture {
    pub fn declare_group(binding: &mut Binding, o: &mut Out) -> TextureBindings {
        let tdiff = binding.next();
        let sdiff = binding.next();

        o.write(Var {
            binding: tdiff,
            uniform: false,
            name: Name::Str("tdiff"),
            ty: Type::TEXTURE2D,
        })
        .write(Var {
            binding: sdiff,
            uniform: false,
            name: Name::Str("sdiff"),
            ty: Type::SAMPLER,
        });

        TextureBindings {
            tdiff: tdiff.get(),
            sdiff: sdiff.get(),
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct TextureBindings {
    pub tdiff: u32,
    pub sdiff: u32,
}

pub(crate) struct Ambient;

impl Ambient {
    pub fn declare_group(binding: &mut Binding, o: &mut Out) -> u32 {
        let binding = binding.next();
        o.write(Var {
            binding,
            uniform: true,
            name: Name::Str("ambient"),
            ty: Type::VEC4,
        });

        binding.get()
    }
}

#[derive(Clone, Copy)]
pub enum View {
    None,
    Camera,
}

impl View {
    pub(crate) fn define_type(self, o: &mut Out) {
        if let Self::Camera = self {
            o.write(Struct {
                name: "Camera",
                fields: vec![Field {
                    location: Location::None,
                    name: Name::Str("view"),
                    ty: Type::MAT4,
                }],
            });
        }
    }

    pub(crate) fn declare_group(self, binding: &mut Binding, o: &mut Out) -> Option<u32> {
        match self {
            Self::None => None,
            Self::Camera => {
                let binding = binding.next();
                o.write(Var {
                    binding,
                    uniform: true,
                    name: Name::Str("camera"),
                    ty: Type::Simple("Camera"),
                });

                Some(binding.get())
            }
        }
    }

    pub(crate) fn calc_view(self, o: &mut Out) {
        o.write_str(match self {
            Self::None => "world",
            Self::Camera => "camera.view * world",
        });
    }
}

#[derive(Clone, Copy)]
pub struct SourceArrays(&'static [SourceArray]);

impl SourceArrays {
    pub const EMPTY: Self = Self(&[]);

    #[must_use]
    pub const fn new(arrays: &'static [SourceArray]) -> Self {
        assert!(
            arrays.len() <= 4,
            "the number of source arrays cannot be greater than 4",
        );

        Self(arrays)
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn define_type(&self, o: &mut Out) {
        if self.is_empty() {
            return;
        }

        o.write(Struct {
            name: "Source",
            fields: vec![
                Field {
                    location: Location::None,
                    name: Name::Str("col"),
                    ty: Type::VEC3,
                },
                Field {
                    location: Location::None,
                    name: Name::Str("rad"),
                    ty: Type::F32,
                },
                Field {
                    location: Location::None,
                    name: Name::Str("pos"),
                    ty: Type::VEC3,
                },
            ],
        })
        .write(Struct {
            name: "Len",
            fields: vec![
                Field {
                    location: Location::None,
                    name: Name::Str("n"),
                    ty: Type::U32,
                },
                Field {
                    location: Location::None,
                    name: Name::Str("pad0"),
                    ty: Type::U32,
                },
                Field {
                    location: Location::None,
                    name: Name::Str("pad1"),
                    ty: Type::U32,
                },
                Field {
                    location: Location::None,
                    name: Name::Str("pad2"),
                    ty: Type::U32,
                },
            ],
        });
    }

    pub(crate) fn declare_group(&self, binding: &mut Binding, o: &mut Out) -> Vec<SourceBindings> {
        self.enumerate()
            .map(|(n, SourceArray { size, .. })| SourceBindings {
                binding_array: {
                    let binding = binding.next();
                    o.write(Var {
                        binding,
                        uniform: true,
                        name: Name::Num {
                            str: "sources_array",
                            n,
                        },
                        ty: Type::Array {
                            ty: &Type::Simple("Source"),
                            size,
                        },
                    });

                    binding.get()
                },
                binding_len: {
                    let binding = binding.next();
                    o.write(Var {
                        binding,
                        uniform: true,
                        name: Name::Num {
                            str: "sources_len",
                            n,
                        },
                        ty: Type::Simple("Len"),
                    });

                    binding.get()
                },
                size,
            })
            .collect()
    }

    fn enumerate(&self) -> impl Iterator<Item = (u32, SourceArray)> {
        (0..).zip(self.0.iter().copied())
    }
}

#[derive(Clone, Copy)]
pub struct SourceArray {
    kind: SourceKind,
    size: u8,
}

impl SourceArray {
    #[must_use]
    pub const fn new(kind: SourceKind, size: u8) -> Self {
        assert!(size != 0, "source array cannot have size equal to zero");
        assert!(size <= 127, "source array cannot be larger than 127");
        Self { kind, size }
    }
}

#[derive(Clone, Copy)]
pub enum SourceKind {
    Glow,
    Gloom,
}

#[derive(Clone, Copy)]
pub struct SourceBindings {
    pub binding_array: u32,
    pub binding_len: u32,
    pub size: u8,
}

#[derive(Clone, Copy)]
pub struct LightSpaces(&'static [SpaceKind]);

impl LightSpaces {
    pub const EMPTY: Self = Self(&[]);

    #[must_use]
    pub const fn new(spaces: &'static [SpaceKind]) -> Self {
        assert!(
            spaces.len() <= 4,
            "the number of light spaces cannot be greater than 4",
        );

        Self(spaces)
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn define_type(&self, o: &mut Out) {
        if self.is_empty() {
            return;
        }

        o.write(Struct {
            name: "Space",
            fields: vec![
                Field {
                    location: Location::None,
                    name: Name::Str("model"),
                    ty: Type::MAT4,
                },
                Field {
                    location: Location::None,
                    name: Name::Str("col"),
                    ty: Type::VEC3,
                },
            ],
        });
    }

    pub(crate) fn declare_group(&self, binding: &mut Binding, o: &mut Out) -> SpaceBindings {
        if self.is_empty() {
            return SpaceBindings::default();
        }

        SpaceBindings {
            spaces: {
                let binding = binding.next();
                o.write(Var {
                    binding,
                    uniform: true,
                    name: Name::Str("spaces"),
                    ty: Type::Array {
                        ty: &Type::Simple("Space"),
                        size: self.len() as u8,
                    },
                });

                binding.get()
            },
            tdiffs: self
                .enumerate()
                .map(|(n, ..)| {
                    let binding = binding.next();
                    o.write(Var {
                        binding,
                        uniform: false,
                        name: Name::Num {
                            str: "space_tdiff",
                            n,
                        },
                        ty: Type::TEXTURE3D,
                    });

                    binding.get()
                })
                .collect(),
            sdiff: {
                let binding = binding.next();
                o.write(Var {
                    binding,
                    uniform: false,
                    name: Name::Str("space_sdiff"),
                    ty: Type::SAMPLER,
                });

                binding.get()
            },
        }
    }

    pub fn enumerate(&self) -> impl Iterator<Item = (u32, SpaceKind)> {
        (0..).zip(self.0.iter().copied())
    }
}

#[derive(Clone, Copy)]
pub enum SpaceKind {
    Rgba,
    Gray,
}

impl SpaceKind {
    fn calc(self, name: &str, index: u32, o: &mut Out) {
        o.write_str("var ")
            .write_str(name)
            .write_str(" = textureSampleLevel(space_tdiff_")
            .write(index)
            .write_str(", space_sdiff, out.space_")
            .write(index)
            .write_str(", 0.);\n    ")
            .write_str("space = ");

        if index == 0 {
            o.write_str(name)
                .write_str(match self {
                    Self::Rgba => ".rgb",
                    Self::Gray => ".rrr",
                })
                .write_str(" * spaces[")
                .write(index)
                .write_str("].col;\n    ");
        } else {
            o.write_str("max(space, ")
                .write_str(name)
                .write_str(match self {
                    Self::Rgba => ".rgb",
                    Self::Gray => ".rrr",
                })
                .write_str(" * spaces[")
                .write(index)
                .write_str("].col);\n    ");
        }
    }
}

#[derive(Clone, Default)]
pub struct SpaceBindings {
    pub spaces: u32,
    pub tdiffs: Vec<u32>,
    pub sdiff: u32,
}

impl SpaceBindings {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tdiffs.is_empty()
    }
}
