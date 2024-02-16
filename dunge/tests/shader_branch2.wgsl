struct VertexOutput {
    @builtin(position) member: vec4<f32>,
}

@vertex 
fn vs() -> VertexOutput {
    var local: vec4<f32>;

    if false {
        local = vec4<f32>(4f, 4f, 4f, 4f);
    } else {
        if true {
            local = vec4<f32>(3f, 3f, 3f, 3f);
        } else {
            if true {
                local = vec4<f32>(2f, 2f, 2f, 2f);
            } else {
                local = vec4<f32>(1f, 1f, 1f, 1f);
            }
        }
    }
    let _e12: vec4<f32> = local;
    return VertexOutput(_e12);
}

@fragment 
fn fs(param: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1f, 1f, 1f, 1f);
}
