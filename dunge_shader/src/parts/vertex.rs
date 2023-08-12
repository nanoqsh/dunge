use crate::{
    nodes::*,
    out::Out,
    parts::{
        color::Color,
        sources::{SourceArrays, SourceKind},
        spaces::LightSpaces,
        textures::TexturesNumber,
        view::ViewKind,
    },
};

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

#[derive(Clone, Copy)]
pub enum Dimension {
    D2,
    D3,
}

pub(crate) struct VertexOutput {
    pub fragment: Fragment,
    pub static_color: Option<Color>,
    pub ambient: bool,
    pub textures: TexturesNumber,
    pub source_arrays: SourceArrays,
    pub light_spaces: LightSpaces,
    pub instance_colors: bool,
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

        if self.instance_colors {
            fields.push(Field {
                location: location.next(),
                name: Name::Str("instcol"),
                ty: Type::VEC3,
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

    pub fn calc_vertex(&self, input: &VertexInput, camera: ViewKind, o: &mut Out) {
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

        if self.instance_colors {
            o.write_str("    out.instcol = instcol.col;\n");
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
        self.textures.calc_fragment(o);
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

        if self.textures.has_textures() {
            col.out().write_str("tex.rgb");
        }

        if self.instance_colors {
            col.out().write_str("out.instcol");
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

            for (n, array) in self.source_arrays.enumerate() {
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

                match array.kind() {
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
