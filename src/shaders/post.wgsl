struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) map: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    switch in_vertex_index {
        case 0u {
            out.clip_position = vec4(1.0, -1.0, 0.0, 1.0);
            out.map = vec2(1.0, 1.0);
        }
        case 1u {
            out.clip_position = vec4(-1.0, -1.0, 0.0, 1.0);
            out.map = vec2(0.0, 1.0);
        }
        case 2u {
            out.clip_position = vec4(1.0, 1.0, 0.0, 1.0);
            out.map = vec2(1.0, 0.0);
        }
        case 3u {
            out.clip_position = vec4(-1.0, 1.0, 0.0, 1.0);
            out.map = vec2(0.0, 0.0);
        }
        default {}
    }
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.map);
}
