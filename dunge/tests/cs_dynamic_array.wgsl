@group(0) @binding(0) 
var<storage> global: array<f32>;

@compute @workgroup_size(64, 1, 1) 
fn cs() {
}
