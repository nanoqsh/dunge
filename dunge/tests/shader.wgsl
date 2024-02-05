struct VertexOutput {
    @builtin(position) member: vec4<f32>,
}

@vertex 
fn vs() -> VertexOutput {
    let _e7: mat2x2<f32> = -(mat2x2<f32>(vec2<f32>(1f, 0f), vec2<f32>(0f, 1f)));
    return VertexOutput(((vec4<f32>(_e7[0], (_e7[0] + -(_e7[1]))) * f32(1i)) * vec3<f32>(1f, 1f, 1f).z));
}

@fragment 
fn fs(param: VertexOutput) -> @location(0) vec4<f32> {
    return (vec4<f32>(0f, 0f, 1f, 1f) + vec4<f32>(0f, 0f, 0f, 0f));
}
