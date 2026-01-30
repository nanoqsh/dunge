// vert  0: Vec2(Float32)
// vert  1: Vec3(Float32)

struct type_2 {
    @location(0) pos: vec2<f32>,
    @location(1) col: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) col: vec3<f32>,
}

@vertex 
fn vs(param: type_2) -> VertexOutput {
    return VertexOutput(vec4<f32>(param.pos, vec2<f32>(0f, 1f)), param.col);
}

@fragment 
fn fs(param_1: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(param_1.col, 1f);
}
