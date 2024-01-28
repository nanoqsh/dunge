struct type_1 {
    @location(0) member: vec2<f32>,
    @location(1) member_1: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) member: vec4<f32>,
    @location(0) member_1: vec2<f32>,
}

@group(0) @binding(0) 
var global: texture_2d<f32>;
@group(0) @binding(1) 
var global_1: sampler;

@vertex 
fn vs(param: type_1) -> VertexOutput {
    return VertexOutput(vec4<f32>(param.member, vec2<f32>(0f, 1f)), param.member_1);
}

@fragment 
fn fs(param_1: VertexOutput) -> @location(0) vec4<f32> {
    let _e4: vec4<f32> = textureSample(global, global_1, param_1.member_1);
    return _e4;
}
