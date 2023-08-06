use crate::{nodes::*, out::Out};

/// Light source arrays. Described by a slice of [source arrays](SourceArray).
#[derive(Clone, Copy)]
pub struct SourceArrays(&'static [SourceArray]);

impl SourceArrays {
    /// Empty light sources.
    pub const EMPTY: Self = Self(&[]);

    /// Creates a new [`SourceArrays`] from the slice of [`SourceArray`].
    ///
    /// # Panics
    /// Panic if the number of source arrays exceeds the maximum allowed.
    pub const fn new(arrays: &'static [SourceArray]) -> Self {
        assert!(
            arrays.len() <= 4,
            "the number of source arrays cannot be greater than 4",
        );

        Self(arrays)
    }

    /// Returns the length of light source arrays.
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Checks if the light source arrays is empty.
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

    pub(crate) fn enumerate(&self) -> impl Iterator<Item = (u32, SourceArray)> {
        (0..).zip(self.0.iter().copied())
    }
}

/// The light source array.
#[derive(Clone, Copy)]
pub struct SourceArray {
    kind: SourceKind,
    size: u8,
}

impl SourceArray {
    /// Creates a new [`SourceArray`].
    ///
    /// # Panics
    /// This function will panic if the source array have a zero size or is larger than 127.
    pub const fn new(kind: SourceKind, size: u8) -> Self {
        assert!(size != 0, "source array cannot have size equal to zero");
        assert!(size <= 127, "source array cannot be larger than 127");
        Self { kind, size }
    }

    pub(crate) fn kind(self) -> SourceKind {
        self.kind
    }
}

/// The light sources kind. Describes the nature of light.
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
