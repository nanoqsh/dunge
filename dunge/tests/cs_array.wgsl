@group(0) @binding(0) 
var<storage> global: array<f32, 4>;

@compute @workgroup_size(64, 1, 1) 
fn cs(@builtin(global_invocation_id) param: vec3<u32>) {
}
