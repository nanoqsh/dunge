struct VertexOutput {
    @builtin(position) member: vec4<f32>,
}

@vertex 
fn vs(@builtin(vertex_index) param: u32) -> VertexOutput {
    let _e1: f32 = f32(param);
    return VertexOutput(vec4<f32>(_e1, _e1, _e1, _e1));
}

@fragment 
fn fs(param_1: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1f, 1f, 1f, 1f);
}
