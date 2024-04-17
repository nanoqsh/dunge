struct VertexOutput {
    @builtin(position) member: vec4<f32>,
}

@vertex 
fn vs() -> VertexOutput {
    return VertexOutput(vec4<f32>(1f, 1f, 1f, 1f));
}

@fragment 
fn fs(param: VertexOutput) -> @location(0) vec4<f32> {
    var local: vec4<f32>;

    if true {
        discard;
    } else {
        local = vec4<f32>(1f, 1f, 1f, 1f);
    }
    let _e5: vec4<f32> = local;
    return _e5;
}
