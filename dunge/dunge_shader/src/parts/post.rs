use crate::out::Out;

pub(crate) struct Post {
    pub antialiasing: bool,
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
    }
}
