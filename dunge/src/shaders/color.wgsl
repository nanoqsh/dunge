struct Camera {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) col: vec3<f32>,
}

struct InstanceInput {
    @location(2) row0: vec4<f32>,
    @location(3) row1: vec4<f32>,
    @location(4) row2: vec4<f32>,
    @location(5) row3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) col: vec3<f32>,
    @location(1) world: vec3<f32>,
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
    out.col = vert.col;
    out.world = world.xyz;
    return out;
}

@group(1) @binding(0)
var<uniform> ambient: vec4<f32>;
@group(1) @binding(1)
var<uniform> sources: Sources;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let light = ambient.rgb + diffuse_light(in.world);
    return vec4(light * in.col, 1.);
}
