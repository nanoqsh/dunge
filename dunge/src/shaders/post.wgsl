struct Data {
    size: vec2<f32>,
    factor: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> data: Data;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) map: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    switch in_vertex_index {
        case 0u {
            out.clip_position = vec4(1., -1., 0., 1.);
            out.map = data.factor.xy;
        }
        case 1u {
            out.clip_position = vec4(1., 1., 0., 1.);
            out.map = vec2(data.factor.x, 0.);
        }
        case 2u {
             out.clip_position = vec4(-1., -1., 0., 1.);
            out.map = vec2(0., data.factor.y);
        }
        case 3u {
            out.clip_position = vec4(-1., 1., 0., 1.);
            out.map = vec2(0., 0.);
        }
        default {}
    }
    return out;
}

@group(1) @binding(0)
var tdiff: texture_2d<f32>;
@group(1) @binding(1)
var sdiff: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(tdiff, sdiff, in.map);
}
