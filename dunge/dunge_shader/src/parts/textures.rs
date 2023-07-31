use crate::{nodes::*, out::Out};

/// Sets the number of texture maps in the shader.
#[derive(Clone, Copy)]
#[must_use]
pub struct TexturesNumber {
    n: u8,
    discard: Option<f32>,
}

impl TexturesNumber {
    pub const N0: Self = Self::new(0);
    pub const N1: Self = Self::new(1);
    pub const N2: Self = Self::new(2);
    pub const N3: Self = Self::new(3);
    pub const N4: Self = Self::new(4);

    const fn new(n: u8) -> Self {
        Self { n, discard: None }
    }

    /// Discard a pixel if its alpha value is less than the specified.
    pub const fn with_discard_threshold(mut self, value: f32) -> Self {
        self.discard = Some(value);
        self
    }

    /// Returns the number of texture maps.
    #[must_use]
    pub const fn len(self) -> usize {
        self.n as usize
    }

    /// Checks if the number of texture maps is zero.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.n == 0
    }

    pub(crate) fn has_textures(self) -> bool {
        !self.is_empty()
    }

    pub(crate) fn declare_group(self, binding: &mut Binding, o: &mut Out) -> TextureBindings {
        if !self.has_textures() {
            return TextureBindings::default();
        }

        let mut tmaps = Vec::with_capacity(self.n as usize);
        for n in 0..self.n as u32 {
            let binding = binding.next();
            tmaps.push(binding.get());
            o.write(Var {
                binding,
                uniform: false,
                name: Name::Num { str: "tmap", n },
                ty: Type::TEXTURE2D,
            });
        }

        let smap = binding.next();
        o.write(Var {
            binding: smap,
            uniform: false,
            name: Name::Str("smap"),
            ty: Type::SAMPLER,
        });

        TextureBindings {
            tmaps,
            smap: smap.get(),
        }
    }

    pub(crate) fn calc_fragment(self, o: &mut Out) {
        match self.n {
            0 => return,
            1 => _ = o.write("let tex = textureSample(tmap_0, smap, out.map);\n    "),
            num => {
                o.write_str("var tex = vec4(0.);\n    ");
                for n in 0..num as u32 {
                    o.write_str("let tex_")
                        .write(n)
                        .write_str(" = textureSample(tmap_")
                        .write(n)
                        .write_str(", smap, out.map);\n    ")
                        .write_str("tex = mix(tex, tex_")
                        .write(n)
                        .write_str(", tex_")
                        .write(n)
                        .write_str(".a);\n    ");
                }
            }
        }

        if let Some(value) = self.discard {
            o.write_str("if tex.a < ")
                .write_f32(value)
                .write_str(" { discard; }\n    ");
        }
    }
}

#[derive(Clone, Default)]
pub struct TextureBindings {
    pub tmaps: Vec<u32>,
    pub smap: u32,
}
