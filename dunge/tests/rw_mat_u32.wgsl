@group(0) @binding(0) 
var<storage> global: mat3x3<f32>;
@group(1) @binding(0) 
var<storage, read_write> global_1: mat3x3<f32>;

@compute @workgroup_size(1, 1, 1) 
fn cs() {
    let _e4: vec3<f32> = global[0];
    global_1[0] = _e4;
}
