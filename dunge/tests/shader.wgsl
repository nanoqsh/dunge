struct VertexOutput {
    @builtin(position) member: vec4<f32>,
}

@vertex 
fn vs() -> VertexOutput {
    return VertexOutput(((vec4<f32>(1f, 1f, 1f, 1f) * f32(1i)) * 1f));
}

@fragment 
fn fs(param: VertexOutput) -> @location(0) vec4<f32> {
    return (vec4<f32>(0f, 0f, 1f, 1f) + vec4<f32>(0f, 0f, 0f, 0f));
}
