[[types]]
[[groups]]

@vertex
fn vs_main(input: VertexInput, inst: InstanceInput) -> VertexOutput {
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
fn fs_main(out: VertexOutput) -> @location(0) vec4<f32> {
    var col: vec3<f32>;
    [[fragment_col]]
    return vec4(col, 1.);
}
