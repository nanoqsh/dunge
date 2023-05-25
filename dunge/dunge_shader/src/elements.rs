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
                    name,
                    ty: Type::VEC4,
                })
                .collect(),
        });
    }
}

#[derive(Clone, Copy)]
pub(crate) struct VertexInput {
    pub fragment: Fragment,
    pub pos: Dimension,
}

impl VertexInput {
    pub fn define_type(self, location: &mut Location, o: &mut Out) {
        let mut fields = vec![Field {
            location: location.next(),
            name: "pos",
            ty: match self.pos {
                Dimension::D2 => Type::VEC2,
                Dimension::D3 => Type::VEC3,
            },
        }];

        if self.fragment.vertex_color {
            fields.push(Field {
                location: location.next(),
                name: "col",
                ty: Type::VEC3,
            });
        }

        o.write(Struct {
            name: "VertexInput",
            fields,
        });
    }

    pub fn calc_world(self, o: &mut Out) {
        o.write_str(match self.pos {
            Dimension::D2 => "model * vec4(input.pos, 0., 1.)",
            Dimension::D3 => "model * vec4(input.pos, 1.)",
        });
    }
}

#[derive(Clone, Copy)]
pub(crate) struct VertexOutput {
    pub fragment: Fragment,
    pub base_color: Option<Color>,
    pub world: bool,
}

impl VertexOutput {
    pub fn define_type(self, location: &mut Location, o: &mut Out) {
        let mut fields = vec![Field {
            location: Location::Position,
            name: "pos",
            ty: Type::VEC4,
        }];

        if self.fragment.vertex_color {
            fields.push(Field {
                location: location.next(),
                name: "col",
                ty: Type::VEC3,
            });
        }

        if self.world {
            fields.push(Field {
                location: location.next(),
                name: "world",
                ty: Type::VEC3,
            });
        }

        o.write(Struct {
            name: "VertexOutput",
            fields,
        });
    }

    pub fn calc_vertex(self, input: VertexInput, camera: Camera, o: &mut Out) {
        o.write_str("let world = ");
        input.calc_world(o);
        o.write_str(";\n");

        o.write_str("    out.pos = ");
        camera.calc_view(o);
        o.write_str(";\n");

        if self.fragment.vertex_color {
            o.write_str("    out.col = input.col;\n");
        }

        if self.world {
            o.write_str("    out.world = world.xyz;\n");
        }
    }

    pub fn calc_fragment(self, o: &mut Out) {
        if self.fragment.vertex_texture {
            o.write_str(
                "let tex = textureSample(tdiff, sdiff, out.map); \n    \
                if tex.w < 0.95 { \
                    discard; \
                } \n    ",
            );
        }

        let mut mult = o.write_str("col = ").separated(" * ");
        if let Some(Color { r, g, b }) = self.base_color {
            mult.out()
                .write_str("vec3(")
                .write_f32(r)
                .write_str(", ")
                .write_f32(g)
                .write_str(", ")
                .write_f32(b)
                .write_str(")");
        }

        if self.fragment.vertex_color {
            mult.out().write_str("out.col");
        }

        if self.fragment.vertex_texture {
            mult.out().write_str("tex.rgb");
        }

        mult.write_default("vec3(0.)");
        o.write_str(";\n");
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

#[derive(Clone, Copy)]
pub(crate) struct Texture;

impl Texture {
    pub fn declare_group(binding: &mut Binding, o: &mut Out) -> TextureBindings {
        let tdiff = binding.next();
        let sdiff = binding.next();

        o.write(Var {
            binding: tdiff,
            uniform: false,
            name: "tdiff",
            ty: Type::TEXTURE2D,
        })
        .write(Var {
            binding: sdiff,
            uniform: false,
            name: "sdiff",
            ty: Type::SAMPLER,
        });

        TextureBindings {
            tdiff: tdiff.get(),
            sdiff: sdiff.get(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct TextureBindings {
    pub tdiff: u32,
    pub sdiff: u32,
}

#[derive(Clone, Copy)]
pub enum Camera {
    None,
    View,
}

impl Camera {
    pub(crate) fn define_type(self, o: &mut Out) {
        if let Self::View = self {
            o.write(Struct {
                name: "Camera",
                fields: vec![Field {
                    location: Location::None,
                    name: "view",
                    ty: Type::MAT4,
                }],
            });
        }
    }

    pub(crate) fn declare_group(self, binding: &mut Binding, o: &mut Out) -> Option<u32> {
        match self {
            Self::None => None,
            Self::View => {
                let binding = binding.next();
                o.write(Var {
                    binding,
                    uniform: true,
                    name: "camera",
                    ty: Type("Camera"),
                });

                Some(binding.get())
            }
        }
    }

    pub(crate) fn calc_view(self, o: &mut Out) {
        o.write_str(match self {
            Self::None => "world",
            Self::View => "camera.view * world",
        });
    }
}
