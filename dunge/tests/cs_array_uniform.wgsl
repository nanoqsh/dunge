@group(0) @binding(0) 
var<uniform> global: array<f32, 4>;

@compute @workgroup_size(64, 1, 1) 
fn cs() {
}
