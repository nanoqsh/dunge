struct type_2 {
    @location(2) member: vec4<f32>,
    @location(3) member_1: vec4<f32>,
    @location(4) member_2: vec4<f32>,
    @location(5) member_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) member: vec4<f32>,
}

@vertex 
fn vs(@location(0) param: f32, @location(1) param_1: vec4<f32>, param_2: type_2) -> VertexOutput {
    return VertexOutput(((mat4x4<f32>(param_2.member, param_2.member_1, param_2.member_2, param_2.member_3) * param_1) * param));
}

@fragment 
fn fs(param_3: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1f, 1f, 1f, 1f);
}
