[[types]]
[[groups]]

@vertex
fn vsmain(inst: InstanceInput,[[instance_color]] input: VertexInput) -> VertexOutput {
    let model = mat4x4<f32>(
        inst.r0,
        inst.r1,
        inst.r2,
        inst.r3,
    );

    var out: VertexOutput;
    [[vertex_out]]
    return out;
}

@fragment
fn fsmain(out: VertexOutput) -> @location(0) vec4<f32> {
    var col: vec3<f32>;
    [[fragment_col]]
    return vec4(col, 1.);
}
