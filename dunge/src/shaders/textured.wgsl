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

    switch n_spaces {
        case 0u {}
        case 1u {
            out.space0 = (spaces[0].model * world).xzy;
        }
        case 2u {
            out.space0 = (spaces[0].model * world).xzy;
            out.space1 = (spaces[1].model * world).xzy;
        }
        case 3u {
            out.space0 = (spaces[0].model * world).xzy;
            out.space1 = (spaces[1].model * world).xzy;
            out.space2 = (spaces[2].model * world).xzy;
        }
        case 4u {
            out.space0 = (spaces[0].model * world).xzy;
            out.space1 = (spaces[1].model * world).xzy;
            out.space2 = (spaces[2].model * world).xzy;
            out.space3 = (spaces[3].model * world).xzy;
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
var<uniform> ambient: vec3<f32>;
@group(2) @binding(1)
var<uniform> sources: array<Source, 64>;
@group(2) @binding(2)
var<uniform> n_sources: u32;

struct Space {
    model: mat4x4<f32>,
    col: vec3<f32>,
    flags: u32,
}

@group(3) @binding(0)
var<uniform> spaces: array<Space, 4>;
@group(3) @binding(1)
var<uniform> n_spaces: u32;
@group(3) @binding(2)
var space0_tdiff: texture_3d<f32>;
@group(3) @binding(3)
var space1_tdiff: texture_3d<f32>;
@group(3) @binding(4)
var space2_tdiff: texture_3d<f32>;
@group(3) @binding(5)
var space3_tdiff: texture_3d<f32>;
@group(3) @binding(6)
var space_sdiff: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let out = textureSample(tdiff, sdiff, in.map);
    if out.w < 0.9 {
        discard;
    }

    var space = vec3(0.0);
    switch n_spaces {
        case 0u {}
        case 1u {
            space = textureSampleLevel(space0_tdiff, space_sdiff, in.space0, 0.0).rgb * spaces[0].col;
        }
        case 2u {
            let a = textureSampleLevel(space0_tdiff, space_sdiff, in.space0, 0.0).rgb * spaces[0].col;
            let b = textureSampleLevel(space1_tdiff, space_sdiff, in.space1, 0.0).rgb * spaces[1].col;
            space = max(a, b);
        }
        case 3u {
            let a = textureSampleLevel(space0_tdiff, space_sdiff, in.space0, 0.0).rgb * spaces[0].col;
            let b = textureSampleLevel(space1_tdiff, space_sdiff, in.space1, 0.0).rgb * spaces[1].col;
            let c = textureSampleLevel(space2_tdiff, space_sdiff, in.space2, 0.0).rgb * spaces[2].col;
            space = max(a, max(b, c));
        }
        case 4u {
            let a = textureSampleLevel(space0_tdiff, space_sdiff, in.space0, 0.0).rgb * spaces[0].col;
            let b = textureSampleLevel(space1_tdiff, space_sdiff, in.space1, 0.0).rgb * spaces[1].col;
            let c = textureSampleLevel(space2_tdiff, space_sdiff, in.space2, 0.0).rgb * spaces[2].col;
            let d = textureSampleLevel(space3_tdiff, space_sdiff, in.space3, 0.0).rgb * spaces[3].col;
            space = max(max(a, b), max(c, d));
        }
        default {}
    }
    
    let light = ambient + diffuse_light(in.world) + space;
    return vec4(light * out.rgb, out.a);
}
