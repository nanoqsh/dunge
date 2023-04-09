struct Source {
    pos: vec3<f32>,
    rad: f32,
    col: vec3<f32>,
    flags: u32,
}

fn light(world: vec3<f32>) -> vec3<f32> {
    var diffuse = vec3(0., 0., 0.);
    for (var i: u32 = 0u; i < n_sources; i++) {
        let source = sources[i];

        if world.x > source.pos.x - source.rad && world.x < source.pos.x + source.rad
        && world.y > source.pos.y - source.rad && world.y < source.pos.y + source.rad
        && world.z > source.pos.z - source.rad && world.z < source.pos.z + source.rad {
            let len = length(world - source.pos);
            if len < source.rad {
                var sharp = 1.;
                if (source.flags & 1u) == 0u {
                    sharp -= (len / source.rad);
                }

                var gloom: vec3<f32>;
                if (source.flags & 2u) == 0u {
                    gloom = vec3(1.);
                } else {
                    gloom = -ambient;
                }

                diffuse += gloom * sharp * source.col;
            }
        }
    }

    return (ambient + diffuse);
}
