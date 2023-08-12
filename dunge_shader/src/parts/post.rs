use crate::out::Out;

pub(crate) struct Post {
    pub antialiasing: bool,
    pub vignette: Vignette,
}

impl Post {
    pub fn calc_fragment(&self, o: &mut Out) {
        if self.antialiasing {
            o.write_str(
                "\
                let s0 = vec2(data.step.x, data.step.y);\n    \
                let s1 = vec2(-data.step.x, data.step.y);\n    \
                let s2 = vec2(data.step.x, -data.step.y);\n    \
                let s3 = vec2(-data.step.x, -data.step.y);\n    \
                col = (\n          \
                      textureSample(tmap, smap, in.map + s0)\n        \
                    + textureSample(tmap, smap, in.map + s1)\n        \
                    + textureSample(tmap, smap, in.map + s2)\n        \
                    + textureSample(tmap, smap, in.map + s3)\n    \
                ) * 0.25;\n    ",
            );
        } else {
            o.write_str("col = textureSample(tmap, smap, in.map);\n    ");
        }

        if let Vignette::Color { r, g, b, f } = self.vignette {
            o.write_str("\n    ")
                .write_str("let vcol = vec4(")
                .write_f32(r)
                .write_str(", ")
                .write_f32(g)
                .write_str(", ")
                .write_f32(b)
                .write_str(", 1.);\n    ")
                .write_str("let vforce = length(in.uni * 2. - 1.) * ")
                .write_f32(f)
                .write_str(";\n    ")
                .write_str("col = mix(col, vcol, vforce);\n    ");
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum Vignette {
    #[default]
    None,
    Color {
        r: f32,
        g: f32,
        b: f32,
        f: f32,
    },
}
