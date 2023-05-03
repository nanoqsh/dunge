struct Source {
    pos: vec3<f32>,
    rad: f32,
    col: vec3<f32>,
    flags: u32,
}

struct Sources {
    data: array<Source, 64>,
    len: u32,
}

fn diffuse_light(world: vec3<f32>) -> vec3<f32> {
    var diffuse = vec3(0.);
    for (var i = 0u; i < sources.len; i++) {
        let source = sources.data[i];

        if world.x > source.pos.x - source.rad && world.x < source.pos.x + source.rad
        && world.y > source.pos.y - source.rad && world.y < source.pos.y + source.rad
        && world.z > source.pos.z - source.rad && world.z < source.pos.z + source.rad {
            let len = length(world - source.pos);
            if len < source.rad {
                var sharp = 1.;
                if (source.flags & 1u) == 0u {
                    let e = len / source.rad;
                    sharp -= e * e;
                }

                var gloom: vec3<f32>;
                if (source.flags & 2u) == 0u {
                    gloom = vec3(1.);
                } else {
                    gloom = -ambient.rgb;
                }

                diffuse += gloom * sharp * source.col;
            }
        }
    }

    return diffuse;
}
