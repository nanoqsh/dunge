struct Vert {
    @location(0) pos: vec2f,
    @location(1) col: vec3f,
}

struct VertOutput {
    @builtin(position) pos: vec4f,
    @location(0) col: vec3f,
}

@vertex 
fn vs(param: Vert) -> VertOutput {
    return VertOutput(vec4f(param.pos, vec2f(0f, 1f)), param.col);
}

@fragment 
fn fs(out: VertOutput) -> @location(0) vec4f {
    return vec4f(out.col, 1f);
}
