struct VertexOutput {
    @builtin(position) member: vec4<f32>,
    @location(0) member_1: vec3<f32>,
}

@vertex 
fn vs(@location(0) param: vec2<f32>, @location(1) param_1: vec3<f32>, @builtin(vertex_index) param_2: u32) -> VertexOutput {
    let _e5: f32 = ((f32(param_2) * 2.0943952f) + -1.5707964f);
    return VertexOutput(vec4<f32>(((vec2<f32>(cos(_e5), sin(_e5)) * 0.4f) + param), vec2<f32>(0f, 1f)), param_1);
}

@fragment 
fn fs(param_3: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(param_3.member_1, 1f);
}
