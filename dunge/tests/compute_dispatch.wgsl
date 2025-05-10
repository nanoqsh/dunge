@group(0) @binding(0) 
var<storage, read_write> global: array<u32, 256>;

@compute @workgroup_size(16, 1, 1) 
fn cs(@builtin(global_invocation_id) param: vec3<u32>) {
    global[param.x] = param.x;
}
