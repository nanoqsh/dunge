struct Space {
    model: mat4x4<f32>,
    col: vec3<f32>,
    flags: u32,
}

struct Spaces {
    data: array<Space, 4>,
    len: u32,
}

fn space_light(
    space0: vec3<f32>,
    space1: vec3<f32>,
    space2: vec3<f32>,
    space3: vec3<f32>,
) -> vec3<f32> {
    var space = vec3(0.);
    switch spaces.len {
        case 0u {}
        case 1u {
            var a = textureSampleLevel(space0_tdiff, space_sdiff, space0, 0.);
            if (spaces.data[0].flags & 1u) == 0u {
                space = a.rgb * spaces.data[0].col;
            } else {
                space = a.rrr * spaces.data[0].col;
            }
        }
        case 2u {
            var a = textureSampleLevel(space0_tdiff, space_sdiff, space0, 0.);
            if (spaces.data[0].flags & 1u) == 0u {
                space = a.rgb * spaces.data[0].col;
            } else {
                space = a.rrr * spaces.data[0].col;
            }

            var b = textureSampleLevel(space1_tdiff, space_sdiff, space1, 0.);
            if (spaces.data[1].flags & 1u) == 0u {
                space = max(space, b.rgb * spaces.data[1].col);
            } else {
                space = max(space, b.rrr * spaces.data[1].col);
            }
        }
        case 3u {
            var a = textureSampleLevel(space0_tdiff, space_sdiff, space0, 0.);
            if (spaces.data[0].flags & 1u) == 0u {
                space = a.rgb * spaces.data[0].col;
            } else {
                space = a.rrr * spaces.data[0].col;
            }

            var b = textureSampleLevel(space1_tdiff, space_sdiff, space1, 0.);
            if (spaces.data[1].flags & 1u) == 0u {
                space = max(space, b.rgb * spaces.data[1].col);
            } else {
                space = max(space, b.rrr * spaces.data[1].col);
            }

            var c = textureSampleLevel(space2_tdiff, space_sdiff, space2, 0.);
            if (spaces.data[2].flags & 1u) == 0u {
                space = max(space, c.rgb * spaces.data[2].col);
            } else {
                space = max(space, c.rrr * spaces.data[2].col);
            }
        }
        case 4u {
            var a = textureSampleLevel(space0_tdiff, space_sdiff, space0, 0.);
            if (spaces.data[0].flags & 1u) == 0u {
                space = a.rgb * spaces.data[0].col;
            } else {
                space = a.rrr * spaces.data[0].col;
            }

            var b = textureSampleLevel(space1_tdiff, space_sdiff, space1, 0.);
            if (spaces.data[1].flags & 1u) == 0u {
                space = max(space, b.rgb * spaces.data[1].col);
            } else {
                space = max(space, b.rrr * spaces.data[1].col);
            }

            var c = textureSampleLevel(space2_tdiff, space_sdiff, space2, 0.);
            if (spaces.data[2].flags & 1u) == 0u {
                space = max(space, c.rgb * spaces.data[2].col);
            } else {
                space = max(space, c.rrr * spaces.data[2].col);
            }

            var d = textureSampleLevel(space3_tdiff, space_sdiff, space3, 0.);
            if (spaces.data[3].flags & 1u) == 0u {
                space = max(space, d.rgb * spaces.data[3].col);
            } else {
                space = max(space, d.rrr * spaces.data[3].col);
            }
        }
        default {}
    }

    return space;
}
