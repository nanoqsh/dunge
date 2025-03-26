@group(0) @binding(0) 
var<storage, read_write> global: array<f32>;
@group(0) @binding(1) 
var<storage, read_write> global_1: array<atomic<u32>>;

@compute @workgroup_size(64, 1, 1) 
fn cs(@builtin(global_invocation_id) param: vec3<u32>) {
    let _e5: f32 = global[param.x];
    let _e12: u32 = atomicAdd((&global_1[u32((_e5 / 100f))]), u32(1u));
}
