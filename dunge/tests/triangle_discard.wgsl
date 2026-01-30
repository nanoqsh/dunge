// vert  0: Vec2(Float32)
// vert  1: Vec2(Float32)
// group 0: Fragment 
// 	bind 0: Texture { dim: D2, scalar: Float32 }
// 	bind 1: Sampler

struct type_1 {
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) 
var global: texture_2d<f32>;
@group(0) @binding(1) 
var global_1: sampler;

@vertex 
fn vs(param: type_1) -> VertexOutput {
    return VertexOutput(vec4<f32>(param.pos, vec2<f32>(0f, 1f)), param.uv);
}

@fragment 
fn fs(param_1: VertexOutput) -> @location(0) vec4<f32> {
    var local: vec4<f32>;

    let _e4: vec4<f32> = textureSample(global, global_1, param_1.uv);
    local = _e4;
    let _e8: f32 = local.w;
    if (_e8 < 0.9f) {
        discard;
    }
    let _e13: vec4<f32> = local;
    return _e13;
}
