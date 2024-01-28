struct VertexOutput {
    @builtin(position) member: vec4<f32>,
}

@vertex 
fn vs(@builtin(vertex_index) param: u32) -> VertexOutput {
    let _e5: f32 = ((f32(param) * 2.0943952f) + -1.5707964f);
    return VertexOutput(vec4<f32>(cos(_e5), (sin(_e5) + 0.25f), 0f, 1f));
}

@fragment 
fn fs(param_1: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1f, 0f, 0f, 1f);
}
