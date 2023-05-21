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
    pub pos: Dimension,
    pub col: Color,
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

        if self.col.has_color_in_vertex() {
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
pub enum Color {
    Fixed { r: f32, g: f32, b: f32 },
    FromVertex,
}

impl Color {
    pub(crate) fn has_color_in_vertex(self) -> bool {
        matches!(self, Self::FromVertex)
    }
}

#[derive(Clone, Copy)]
pub(crate) struct VertexOutput {
    pub col: Color,
    pub world: bool,
}

impl VertexOutput {
    pub fn define_type(self, location: &mut Location, o: &mut Out) {
        let mut fields = vec![Field {
            location: Location::Position,
            name: "pos",
            ty: Type::VEC4,
        }];

        if self.col.has_color_in_vertex() {
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

        if self.col.has_color_in_vertex() {
            o.write_str("    out.col = input.col;\n");
        }

        if self.world {
            o.write_str("    out.world = world.xyz;\n");
        }
    }

    pub fn calc_fragment(self, o: &mut Out) {
        o.write_str("col = ");
        match self.col {
            Color::Fixed { r, g, b } => {
                o.write_str("vec3(");
                o.write(r);
                o.write_str(", ");
                o.write(g);
                o.write_str(", ");
                o.write(b);
                o.write_str(")");
            }
            Color::FromVertex => o.write_str("out.col"),
        }

        o.write_str(";\n");
    }
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

    pub(crate) fn declare_group(self, binding: &mut Binding, o: &mut Out) {
        if let Self::View = self {
            o.write(Uniform {
                binding: binding.next(),
                name: "camera",
                ty: Type("Camera"),
            });
        }
    }

    pub(crate) fn calc_view(self, o: &mut Out) {
        o.write_str(match self {
            Self::None => "world",
            Self::View => "camera.view * world",
        });
    }
}
