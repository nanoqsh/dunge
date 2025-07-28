@group(0) @binding(0) 
var<storage> global: vec3<f32>;
@group(1) @binding(0) 
var<storage, read_write> global_1: vec3<f32>;

@compute @workgroup_size(1, 1, 1) 
fn cs() {
    let _e4: f32 = global.x;
    global_1.x = _e4;
}
