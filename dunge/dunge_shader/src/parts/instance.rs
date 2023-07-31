use crate::{nodes::*, out::Out};

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

#[derive(Clone, Copy)]
pub(crate) struct InstanceColorInput {
    enable: bool,
}

impl InstanceColorInput {
    pub fn new(enable: bool) -> Self {
        Self { enable }
    }

    pub fn define_type(self, location: &mut Location, o: &mut Out) {
        if self.enable {
            o.write(Struct {
                name: "InstanceColorInput",
                fields: vec![Field {
                    location: location.next(),
                    name: Name::Str("col"),
                    ty: Type::VEC3,
                }],
            });
        }
    }

    pub fn define_input(self, o: &mut Out) {
        if self.enable {
            o.write_str(" instcol: InstanceColorInput,");
        }
    }
}
