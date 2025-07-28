@group(0) @binding(0) 
var<storage> global: array<f32, 4>;
@group(1) @binding(0) 
var<storage, read_write> global_1: array<f32, 4>;

@compute @workgroup_size(1, 1, 1) 
fn cs() {
    let _e6: f32 = global[0u];
    global_1[0u] = _e6;
}
