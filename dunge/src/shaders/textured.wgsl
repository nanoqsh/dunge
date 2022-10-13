struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) map: vec2<f32>,
}

struct InstanceInput {
    @location(5) mat0: vec4<f32>,
    @location(6) mat1: vec4<f32>,
    @location(7) mat2: vec4<f32>,
    @location(8) mat3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) map: vec2<f32>,
}

@vertex
fn vs_main(vert: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model = mat4x4<f32>(
        instance.mat0,
        instance.mat1,
        instance.mat2,
        instance.mat3,
    );

    var out: VertexOutput;
    out.clip_position = camera.view_proj * model * vec4<f32>(vert.pos, 1.0);
    out.map = vert.map;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.map);
}
