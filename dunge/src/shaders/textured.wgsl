struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) map: vec2<f32>,
}

struct InstanceInput {
    @location(2) row0: vec4<f32>,
    @location(3) row1: vec4<f32>,
    @location(4) row2: vec4<f32>,
    @location(5) row3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) map: vec2<f32>,
    @location(1) world: vec3<f32>,
    @location(2) space0: vec3<f32>,
    @location(3) space1: vec3<f32>,
    @location(4) space2: vec3<f32>,
    @location(5) space3: vec3<f32>,
}

@vertex
fn vs_main(vert: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model = mat4x4<f32>(
        instance.row0,
        instance.row1,
        instance.row2,
        instance.row3,
    );

    var out: VertexOutput;
    let world = model * vec4(vert.pos, 1.);
    out.pos = camera.view_proj * world;
    out.map = vert.map;
    out.world = world.xyz;

    switch spaces.len {
        case 0u {}
        case 1u {
            out.space0 = (spaces.data[0].model * world).xzy;
        }
        case 2u {
            out.space0 = (spaces.data[0].model * world).xzy;
            out.space1 = (spaces.data[1].model * world).xzy;
        }
        case 3u {
            out.space0 = (spaces.data[0].model * world).xzy;
            out.space1 = (spaces.data[1].model * world).xzy;
            out.space2 = (spaces.data[2].model * world).xzy;
        }
        case 4u {
            out.space0 = (spaces.data[0].model * world).xzy;
            out.space1 = (spaces.data[1].model * world).xzy;
            out.space2 = (spaces.data[2].model * world).xzy;
            out.space3 = (spaces.data[3].model * world).xzy;
        }
        default {}
    }

    return out;
}

@group(1) @binding(0)
var tdiff: texture_2d<f32>;
@group(1) @binding(1)
var sdiff: sampler;

@group(2) @binding(0)
var<uniform> ambient: vec4<f32>;
@group(2) @binding(1)
var<uniform> sources: Sources;

struct Space {
    model: mat4x4<f32>,
    col: vec3<f32>,
    flags: u32,
}

struct Spaces {
    data: array<Space, 4>,
    len: u32,
}

@group(3) @binding(0)
var<uniform> spaces: Spaces;
@group(3) @binding(1)
var space0_tdiff: texture_3d<f32>;
@group(3) @binding(2)
var space1_tdiff: texture_3d<f32>;
@group(3) @binding(3)
var space2_tdiff: texture_3d<f32>;
@group(3) @binding(4)
var space3_tdiff: texture_3d<f32>;
@group(3) @binding(5)
var space_sdiff: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let out = textureSample(tdiff, sdiff, in.map);
    if out.w < 0.9 {
        discard;
    }

    var space = vec3(0.);
    switch spaces.len {
        case 0u {}
        case 1u {
            var a = textureSampleLevel(space0_tdiff, space_sdiff, in.space0, 0.);
            if (spaces.data[0].flags & 1u) == 0u {
                space = a.rgb * spaces.data[0].col;
            } else {
                space = a.rrr * spaces.data[0].col;
            }
        }
        case 2u {
            var a = textureSampleLevel(space0_tdiff, space_sdiff, in.space0, 0.);
            if (spaces.data[0].flags & 1u) == 0u {
                space = a.rgb * spaces.data[0].col;
            } else {
                space = a.rrr * spaces.data[0].col;
            }

            var b = textureSampleLevel(space1_tdiff, space_sdiff, in.space1, 0.);
            if (spaces.data[1].flags & 1u) == 0u {
                space = max(space, b.rgb * spaces.data[1].col);
            } else {
                space = max(space, b.rrr * spaces.data[1].col);
            }
        }
        case 3u {
            var a = textureSampleLevel(space0_tdiff, space_sdiff, in.space0, 0.);
            if (spaces.data[0].flags & 1u) == 0u {
                space = a.rgb * spaces.data[0].col;
            } else {
                space = a.rrr * spaces.data[0].col;
            }

            var b = textureSampleLevel(space1_tdiff, space_sdiff, in.space1, 0.);
            if (spaces.data[1].flags & 1u) == 0u {
                space = max(space, b.rgb * spaces.data[1].col);
            } else {
                space = max(space, b.rrr * spaces.data[1].col);
            }

            var c = textureSampleLevel(space2_tdiff, space_sdiff, in.space2, 0.);
            if (spaces.data[2].flags & 1u) == 0u {
                space = max(space, c.rgb * spaces.data[2].col);
            } else {
                space = max(space, c.rrr * spaces.data[2].col);
            }
        }
        case 4u {
            var a = textureSampleLevel(space0_tdiff, space_sdiff, in.space0, 0.);
            if (spaces.data[0].flags & 1u) == 0u {
                space = a.rgb * spaces.data[0].col;
            } else {
                space = a.rrr * spaces.data[0].col;
            }

            var b = textureSampleLevel(space1_tdiff, space_sdiff, in.space1, 0.);
            if (spaces.data[1].flags & 1u) == 0u {
                space = max(space, b.rgb * spaces.data[1].col);
            } else {
                space = max(space, b.rrr * spaces.data[1].col);
            }

            var c = textureSampleLevel(space2_tdiff, space_sdiff, in.space2, 0.);
            if (spaces.data[2].flags & 1u) == 0u {
                space = max(space, c.rgb * spaces.data[2].col);
            } else {
                space = max(space, c.rrr * spaces.data[2].col);
            }

            var d = textureSampleLevel(space3_tdiff, space_sdiff, in.space3, 0.);
            if (spaces.data[3].flags & 1u) == 0u {
                space = max(space, d.rgb * spaces.data[3].col);
            } else {
                space = max(space, d.rrr * spaces.data[3].col);
            }
        }
        default {}
    }
    
    let light = ambient.rgb + diffuse_light(in.world) + space;
    return vec4(light * out.rgb, out.a);
}
