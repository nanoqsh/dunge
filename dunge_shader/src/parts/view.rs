use crate::{nodes::*, out::Out};

/// The shader view.
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
