struct type_2 {
    @location(0) member: vec2<f32>,
    @location(1) member_1: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) member: vec4<f32>,
    @location(0) member_1: vec3<f32>,
}

@vertex 
fn vs(param: type_2) -> VertexOutput {
    return VertexOutput(vec4<f32>(param.member, vec2<f32>(0f, 1f)), param.member_1);
}

@fragment 
fn fs(param_1: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(param_1.member_1, 1f);
}
