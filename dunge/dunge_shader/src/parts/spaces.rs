use crate::{nodes::*, out::Out};

/// Light spaces. Described by a slice of [space kinds](SpaceKind).
#[derive(Clone, Copy)]
pub struct LightSpaces(&'static [SpaceKind]);

impl LightSpaces {
    /// Empty light spaces.
    pub const EMPTY: Self = Self(&[]);

    /// Creates a new [`LightSpaces`] from the slice of [`SpaceKind`].
    ///
    /// # Panics
    /// Panic if the number of spaces exceeds the maximum allowed.
    #[must_use]
    pub const fn new(spaces: &'static [SpaceKind]) -> Self {
        assert!(
            spaces.len() <= 4,
            "the number of light spaces cannot be greater than 4",
        );

        Self(spaces)
    }

    /// Returns the length of light space.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks if the light space is empty.
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
            tspaces: self
                .enumerate()
                .map(|(n, ..)| {
                    let binding = binding.next();
                    o.write(Var {
                        binding,
                        uniform: false,
                        name: Name::Num { str: "tspace", n },
                        ty: Type::TEXTURE3D,
                    });

                    binding.get()
                })
                .collect(),
            sspace: {
                let binding = binding.next();
                o.write(Var {
                    binding,
                    uniform: false,
                    name: Name::Str("sspace"),
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

/// The light spaces kind. Describes the number of color components in the space.
#[derive(Clone, Copy)]
pub enum SpaceKind {
    Rgba,
    Gray,
}

impl SpaceKind {
    pub(crate) fn calc(self, name: &str, index: u32, o: &mut Out) {
        o.write_str("var ")
            .write_str(name)
            .write_str(" = textureSampleLevel(tspace_")
            .write(index)
            .write_str(", sspace, out.space_")
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
    pub tspaces: Vec<u32>,
    pub sspace: u32,
}

impl SpaceBindings {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tspaces.is_empty()
    }
}
