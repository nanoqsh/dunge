struct Vert {
    @location(0) pos: vec2f,
    @location(1) col: vec3f,
}

struct Output {
    @builtin(position) pos: vec4f,
    @location(0) col: vec3f,
}

@vertex 
fn vs(v: Vert) -> Output {
    return Output(vec4f(v.pos, vec2f(0f, 1f)), v.col);
}

@fragment 
fn fs(o: Output) -> @location(0) vec4f {
    return vec4f(o.col, 1f);
}
