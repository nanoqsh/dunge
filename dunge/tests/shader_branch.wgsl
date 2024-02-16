struct VertexOutput {
    @builtin(position) member: vec4<f32>,
}

@vertex 
fn vs() -> VertexOutput {
    var local: vec4<f32>;

    if true {
        local = vec4<f32>(1f, 1f, 1f, 1f);
    } else {
        if false {
            local = vec4<f32>(2f, 2f, 2f, 2f);
        } else {
            local = vec4<f32>(3f, 3f, 3f, 3f);
        }
    }
    let _e9: vec4<f32> = local;
    return VertexOutput(_e9);
}

@fragment 
fn fs(param: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1f, 1f, 1f, 1f);
}
