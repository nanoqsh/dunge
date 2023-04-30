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
    @location(2) light_space: vec3<f32>,
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
    out.light_space = (space.model * world).xyz;
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
var<uniform> space: Space;
@group(3) @binding(1)
var space_tdiff: texture_3d<f32>;
@group(3) @binding(2)
var space_sdiff: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let out = textureSample(tdiff, sdiff, in.map);
    if out.w < 0.9 {
        discard;
    }

    let space_light = textureSampleLevel(space_tdiff, space_sdiff, in.light_space.xzy, 0.0).rgb;
    let result = (light(in.world) + space_light * space.col) * out.rgb;
    return vec4(result, out.a);
}
