struct VertexOutput {
    @builtin(position) member: vec4<f32>,
}

@group(0) @binding(0) 
var<storage> global: array<f32, 4>;

@vertex 
fn vs(@builtin(vertex_index) param: u32) -> VertexOutput {
    let _e5: f32 = global[param];
    return VertexOutput((vec4<f32>(1f, 1f, 1f, 1f) * _e5));
}

@fragment 
fn fs(param_1: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1f, 1f, 1f, 1f);
}
