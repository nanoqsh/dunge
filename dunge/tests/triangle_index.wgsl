
struct type_1 {
    @builtin(vertex_index) index: u32,
}

@vertex 
fn vs(param: type_1) -> @builtin(position) vec4<f32> {
    var local: f32;
    var local_1: f32;
    var local_2: f32;
    var local_3: f32;

    local = 2.0943952f;
    local_1 = -1.5707964f;
    local_2 = 0.25f;
    let _e10: f32 = local;
    let _e13: f32 = local_1;
    local_3 = ((f32(param.index) * _e10) + _e13);
    let _e17: f32 = local_3;
    let _e20: f32 = local_2;
    let _e23: f32 = local_3;
    return vec4<f32>(cos(_e23), (sin(_e17) + _e20), 0f, 1f);
}

@fragment 
fn fs() -> @location(0) vec4<f32> {
    return vec4<f32>(1f, 0f, 0f, 1f);
}
