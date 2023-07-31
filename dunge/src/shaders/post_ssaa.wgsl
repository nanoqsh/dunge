struct Data {
    size: vec2<f32>,
    step: vec2<f32>,
    factor: vec2<f32>,
    pad: vec2<u32>,
}

@group(0) @binding(0)
var<uniform> data: Data;

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) map: vec2<f32>,
}

@vertex
fn vsmain(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    switch in_vertex_index {
        case 0u {
            out.pos = vec4(1., -1., 0., 1.);
            out.map = data.factor.xy;
        }
        case 1u {
            out.pos = vec4(1., 1., 0., 1.);
            out.map = vec2(data.factor.x, 0.);
        }
        case 2u {
             out.pos = vec4(-1., -1., 0., 1.);
            out.map = vec2(0., data.factor.y);
        }
        case 3u {
            out.pos = vec4(-1., 1., 0., 1.);
            out.map = vec2(0., 0.);
        }
        default {}
    }
    return out;
}

@group(1) @binding(0)
var tmap: texture_2d<f32>;
@group(1) @binding(1)
var smap: sampler;

@fragment
fn fsmain(in: VertexOutput) -> @location(0) vec4<f32> {
    let s0 = vec2(data.step.x, data.step.y);
    let s1 = vec2(-data.step.x, data.step.y);
    let s2 = vec2(data.step.x, -data.step.y);
    let s3 = vec2(-data.step.x, -data.step.y);
    let col = (
          textureSample(tmap, smap, in.map + s0)
        + textureSample(tmap, smap, in.map + s1)
        + textureSample(tmap, smap, in.map + s2)
        + textureSample(tmap, smap, in.map + s3)
    ) * 0.25;

    return col;
}
