@group(0) @binding(0) 
var<uniform> global: array<array<f32, 4>, 4>;

@compute @workgroup_size(64, 1, 1) 
fn cs() {
}
