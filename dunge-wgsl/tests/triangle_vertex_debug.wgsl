struct Vert {
    @location(0) pos: vec2<f32>,
    @location(1) col: vec3<f32>,
}

struct Output {
    @builtin(position) pos: vec4<f32>,
    @location(0) col: vec3<f32>,
}

@vertex 
fn vs(v: Vert) -> Output {
    return Output(vec4<f32>(v.pos, vec2<f32>(0f, 1f)), v.col);
}

@fragment 
fn fs(o: Output) -> @location(0) vec4<f32> {
    return vec4<f32>(o.col, 1f);
}
