@group(0) @binding(0) 
var<storage, read_write> global: array<f32, 4>;

@compute @workgroup_size(64, 1, 1) 
fn cs() {
    global[0u] = 1f;
}
