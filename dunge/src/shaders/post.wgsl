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
            out.map = vec2(1., 1.);
        }
        case 1u {
            out.clip_position = vec4(-1., -1., 0., 1.);
            out.map = vec2(0., 1.);
        }
        case 2u {
            out.clip_position = vec4(1., 1., 0., 1.);
            out.map = vec2(1., 0.);
        }
        case 3u {
            out.clip_position = vec4(-1., 1., 0., 1.);
            out.map = vec2(0., 0.);
        }
        default {}
    }
    return out;
}

struct Data {
    size: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> data: Data;

@group(1) @binding(0)
var tdiff: texture_2d<f32>;
@group(1) @binding(1)
var sdiff: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // This fixes the annoying effect of a sprite stretching
    // caused by f32 rounding up or down depending on its sign.
    var map = in.map;
    if map.x < 0.5 {
        map.x -= 0.5 / data.size.x;
    }

    return textureSample(tdiff, sdiff, map);
}
