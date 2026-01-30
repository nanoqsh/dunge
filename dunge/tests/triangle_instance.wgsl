// inst  0: Vec2(Float32)
// inst  1: Vec3(Float32)

struct type_2 {
    @location(0) pos: vec2<f32>,
    @location(1) col: vec3<f32>,
}

struct type_4 {
    @builtin(vertex_index) index: u32,
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) col: vec3<f32>,
}

@vertex 
fn vs(param: type_2, param_1: type_4) -> VertexOutput {
    var local: f32;
    var local_1: f32;
    var local_2: f32;
    var local_3: f32;
    var local_4: vec2<f32>;

    local = 0.4f;
    local_1 = 2.0943952f;
    local_2 = -1.5707964f;
    let _e10: f32 = local_1;
    let _e13: f32 = local_2;
    local_3 = ((f32(param_1.index) * _e10) + _e13);
    let _e17: f32 = local_3;
    let _e20: f32 = local_3;
    let _e24: f32 = local;
    local_4 = ((vec2<f32>(cos(_e20), sin(_e17)) * _e24) + param.pos);
    let _e35: vec2<f32> = local_4;
    return VertexOutput(vec4<f32>(_e35, vec2<f32>(0f, 1f)), param.col);
}

@fragment 
fn fs(param_2: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(param_2.col, 1f);
}
